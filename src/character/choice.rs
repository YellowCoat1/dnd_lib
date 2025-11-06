use std::future::Future;
use serde::{Serialize, Deserialize};

/// Represents a tree of possible options that can be *presented* as options that a character can
/// select.
///
/// Each node is either:
/// - `Base(T)`: a single, concrete choice
/// - `Choice(Vec<PresentedOption<T>>)`: a list of sub-options to choose from
///
/// This is used widely throughout the crate. For example, for a class's equipment options
/// or an ability score increase.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PresentedOption<T> {
    Base(T),
    Choice(Vec<PresentedOption<T>>),
}

impl<T> PresentedOption<T> {

    /// Returns the value at `index`.
    /// - If this is a `Base`, returns `Some(self)`
    /// - If this is a `Choice`, returns the child at `index`, or `None` if out of bounds.
    ///
    /// ```
    /// use dnd_lib::character::features::PresentedOption;
    /// let choice = PresentedOption::Choice(vec![
    ///     PresentedOption::Base("a"),
    ///     PresentedOption::Base("b"),
    /// ]);
    /// assert_eq!(choice.choose(1).unwrap(), &PresentedOption::Base("b"));
    /// ```
    pub fn choose(&self, index: usize) -> Option<&PresentedOption<T>> {
        match self {
            PresentedOption::Base(_) => Some(self),
            PresentedOption::Choice(v) => v.get(index),
        }
    }

    /// Replaces this `Choice` with the selected child at the given index.
    ///
    /// Returns `true` if the replacement could be made, or `false` otherwise.
    ///
    /// ```
    /// use dnd_lib::character::features::PresentedOption;
    ///
    /// let mut choice = PresentedOption::Choice(vec![
    ///     PresentedOption::Base("Apples"), 
    ///     PresentedOption::Base("Bananas"), 
    ///     PresentedOption::Base("Oranges")
    /// ]);
    /// choice.choose_in_place(1);
    /// assert_eq!(choice, PresentedOption::Base("Bananas"));
    ///
    /// ```
    pub fn choose_in_place(&mut self, index: usize) -> bool {
        if let PresentedOption::Choice(v) = self {
            if index < v.len() {
                // Take ownership of the chosen child
                let child = v.remove(index);
                *self = child;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Returns the contained value if self is a `Base`, otherwise returns [None].
    pub fn as_base(&self) -> Option<&T> {
        match self {
            PresentedOption::Choice(_) => None,
            PresentedOption::Base(ref t) => Some(t),
        }
    }

    /// Returns the contained value mutably if self is a `Base`, otherwise returns [None].
    pub fn as_base_mut(&mut self) -> Option<&mut T> {
        match self {
            PresentedOption::Choice(_) => None,
            PresentedOption::Base(t) => Some(t)
        }
    }
    
    /// Returns the list of sub-options if this is a `Choice`, otherwise returns [None].
    pub fn choices(&self) -> Option<&[PresentedOption<T>]> {
        match self {
            PresentedOption::Base(_) => None,
            PresentedOption::Choice(v) => Some(v.as_slice()),
        }
    }

    /// Maps a `PresentedOption<T>` to a `PresentedOption<U>`.
    pub fn map<U, F>(self, mut map_closure: F) -> PresentedOption<U> 
    where
        F: FnMut(T) -> U,
    {
        Self::map_inner(self, &mut map_closure)
    }

    fn map_inner<U, F>(this: PresentedOption<T>, map_closure: &mut F) -> PresentedOption<U>
    where
        F: FnMut(T) -> U,
    {
        match this {
            PresentedOption::Base(val) => PresentedOption::Base(map_closure(val)),
            PresentedOption::Choice(children) => PresentedOption::Choice(
                children
                    .into_iter()
                    .map(|child| Self::map_inner(child, map_closure))
                    .collect(),
            )
        }
    }

    /// Maps a `PresentedOption<T>` to a `PresentedOption<U>` within an asynchronous closure.
    pub async fn map_async<'b, U, F, Fut>(self, f: F) -> PresentedOption<U>
    where
        T: 'b,
        U: 'b,
        F: Fn(T) -> Fut + Clone + 'b,
        Fut: Future<Output = U> + 'b,
    {
        //Box::pin(Self::map_async_inner(self, f).await)
        Self::map_async_inner(self, f).await.await

    }

    async fn map_async_inner<'b, U, F, Fut>(
        input: PresentedOption<T>, 
        f: F
    ) -> impl Future<Output = PresentedOption<U>> + 'b
    where
        T: 'b,
        U: 'b,
        F: Fn(T) -> Fut + Clone + 'b,
        Fut: Future<Output = U> + 'b,
    {
        Box::pin(
            async move {
                match input {
                    PresentedOption::Base(val) => {
                        let mapped = f(val).await;
                        PresentedOption::Base(mapped)
                    }
                    PresentedOption::Choice(children) => {
                        let mut mapped_children = Vec::with_capacity(children.len());
                        for child in children {
                            let mapped_child = Self::map_async_inner(child, f.clone()).await.await;
                            mapped_children.push(mapped_child);
                        }
                        PresentedOption::Choice(mapped_children)
                    }
                }
            }
        )
    }


}

impl<T> PresentedOption<Option<T>> {
    /// Converts a `PresentedOption<Option<T>>` to a `Option<PresentedOption<T>>`, discarding
    /// missing values. Useful for API parsing.
    pub fn collect_option(self) -> Option<PresentedOption<T>> {
        match self {
            PresentedOption::Base(Some(v)) => Some(PresentedOption::Base(v)),
            PresentedOption::Base(None) => None,
            PresentedOption::Choice(v) => {
                let mut out =  Vec::with_capacity(v.len());
                for val in v {
                    out.push(val.collect_option()?);
                }
                Some(PresentedOption::Choice(out))
            }
        }
    }
}

impl<T, U> PresentedOption<Result<T, U>> {
    /// Converts a `PresentedOption<Option<T>>` to a `Option<PresentedOption<T>>`, discarding
    /// missing values. Useful for API parsing.
    pub fn collect_result(self) -> Result<PresentedOption<T>, U> {
        match self {
            PresentedOption::Base(Ok(v)) => Ok(PresentedOption::Base(v)),
            PresentedOption::Base(Err(v)) => Err(v),
            PresentedOption::Choice(v) => {
                let mut out =  Vec::with_capacity(v.len());
                for val in v {
                    out.push(val.collect_result()?);
                }
                Ok(PresentedOption::Choice(out))
            }
        }
    }
}

/// Returns references to all [PresentedOption::Base] values within a slice of [PresentedOption]s.
///
/// Only top-level [PresentedOption::Base] values are included.
///
/// ```
/// use dnd_lib::character::features::{PresentedOption, chosen};
///
/// let presented = vec![
///     PresentedOption::Base(1),
///     PresentedOption::Choice(vec![PresentedOption::Base(2)]),
///     PresentedOption::Base(3),
/// ];
///
/// let chosen_options: Vec<&_> = chosen(&presented);
/// assert_eq!(chosen_options, vec![&1, &3]);
/// ```
pub fn chosen<T>(presented: &[PresentedOption<T>]) -> Vec<&T> {
    presented.iter()
        .filter_map(|p| if let PresentedOption::Base(ref f) = p {Some(f)} else {None})
        .collect()
}

/// Returns two lists: One with refrences to all `Base` values, and one with refrences to all
/// `Choice` values.
/// `split(t).0` is equivalent to `chosen(t)`.
pub fn split<T>(presented: &[PresentedOption<T>]) -> (Vec<&T>, Vec<&Vec<PresentedOption<T>>>) {
    let mut chosen = vec![];
    let mut unchosen = vec![];

    for presented_val in presented {
        match presented_val {
            PresentedOption::Base(b) => chosen.push(b),
            PresentedOption::Choice(c) => unchosen.push(c)
        }
    }

    (chosen, unchosen)
}
