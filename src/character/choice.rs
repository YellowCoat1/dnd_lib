use std::future::Future;
use serde::{Serialize, Deserialize};

/// A PresentedOption is a choice between multiple options that can be chosen for a character.
/// This is used widely across the crate, e.g. for choosing a class's items, or for choosing an
/// ability score to increase.
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum PresentedOption<T> {
    Base(T),
    Choice(Vec<PresentedOption<T>>),
}

impl<'a, T> PresentedOption<T> {

    /// Get a refrence to the choice at the index.
    /// If it's just a Base, it returns a refrence to itself.
    pub fn choose(&self, index: usize) -> Option<&PresentedOption<T>> {
        match self {
            PresentedOption::Base(_) => Some(&self),
            PresentedOption::Choice(v) => v.get(index),
        }
    }

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

    pub fn is_base(&self) -> Option<&T> {
        match self {
            PresentedOption::Choice(_) => None,
            PresentedOption::Base(ref t) => Some(t),
        }
    }
    
    /// Gets an array of the choices.
    /// Returns an error if it's a base.
    pub fn choices(&self) -> Result<&[PresentedOption<T>], ()> {
        match self {
            PresentedOption::Base(_) => return Err(()),
            PresentedOption::Choice(v) => Ok(v.as_slice()),
        }
    }

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

/// In a vec of [PresentedOptions](crate::character::features::PresentedOption), get the chosen ones.
///
/// ```
/// use dnd_lib::character::features::{PresentedOption, chosen};
///
/// let presented = vec![
///     PresentedOption::Base(1),
///     PresentedOption::Choice(vec![
///         PresentedOption::Base(2), 
///         PresentedOption::Base(3),
///     ]),
///     PresentedOption::Base(4),
/// ];
///
/// let chosen_options: Vec<&_> = chosen(&presented);
/// assert_eq!(chosen_options, vec![&1, &4]);
/// ```
pub fn chosen<T>(presented: &Vec<PresentedOption<T>>) -> Vec<&T> {
    presented.iter()
        .filter_map(|pf| if let PresentedOption::Base(f) = pf {Some(f)} else {None})
        .collect()
}


