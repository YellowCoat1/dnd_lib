use std::future::Future;
use serde::{Serialize, Deserialize};

/// Represents a tree of possible options that can be *presented* as options that a character can
/// select.
///
/// Each node is either:
/// - [Base(T)]: a single, concrete choice
/// - [Choice(Vec<PresentedOption\<T\>>)]: a list of sub-options to choose from
///
/// This is used widely throughout the crate — for example, to model class equipment options
/// or selectable ability score increases.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PresentedOption<T> {
    Base(T),
    Choice(Vec<PresentedOption<T>>),
}

impl<T> PresentedOption<T> {

    /// Returns a reference to the sub-choice at the index, if it exists.
    /// - If this is a [PresentedOption::Base], returns a reference to itself.
    /// - If this is a [PresentedOption::Choice], returns the child at the provided index, or `None` if out of bounds.
    ///
    /// If it's a [PresentedOption::Base], it returns a reference to itself.
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

    /// Replaces this [PresentedOption::Choice] with the selected child at the given index, modifying it in place.
    ///
    /// Returns `true` if a valid choice was made, or `false` if the index was out of bounds
    /// or the value was already a [PresentedOption::Base].
    ///
    /// ```
    /// use dnd_lib::character::features::PresentedOption;
    ///
    /// let mut choice = PresentedOption::Choice(vec![PresentedOption::Base("Apples"), PresentedOption::Base("Bananas"), PresentedOption::Base("Oranges")]);
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

    /// Returns the list of choices if it's a [PresentedOption::Choice],
    /// otherwise it returns [None].
    pub fn is_base(&self) -> Option<&T> {
        match self {
            PresentedOption::Choice(_) => None,
            PresentedOption::Base(ref t) => Some(t),
        }
    }

    pub fn as_base_mut(&mut self) -> Option<&mut T> {
        match self {
            PresentedOption::Choice(_) => None,
            PresentedOption::Base(t) => Some(t)
        }
    }
    
    /// Gets an array of the choices.
    ///
    /// Returns an error if it's a [PresentedOption::Base].
    pub fn choices(&self) -> Option<&[PresentedOption<T>]> {
        match self {
            PresentedOption::Base(_) => None,
            PresentedOption::Choice(v) => Some(v.as_slice()),
        }
    }

    /// Maps a [PresentedOption] to a different type.
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

    /// Maps a [PresentedOption] to a different type in an async closure.
    ///
    /// Primarily used for parsing api results. This will rarely (if ever) be used by a regular
    /// user.
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
    /// Collects an [Option] out of a [PresentedOption].
    /// 
    /// Primarily used for parsing api results. This will rarely (if ever) be used by a regular
    /// user.
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
    /// Collects a [Result] out of a [PresentedOption].
    ///
    /// Primarily used for parsing api results. This will rarely (if ever) be used by a regular
    /// user.
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


