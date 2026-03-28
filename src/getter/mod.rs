//! Traits and errors for retrieving D&D Character data.
use std::error::Error;

use async_trait::async_trait;

use crate::rules2014::{background::Background, class::Class, items::Item, spells::Spell, Race};

/// A trait representing a source capable of retrieving D&D data, e.g. from an api.
///
/// This trait's definition looks imposing, but most of that is caused by the async implementation.
/// If you're just using an existing getter struct, you can treat each method similarly to
/// something like
/// `get_race(&self, name: &str) -> Result<Race, CharacterDataError>`.
///
/// ## Implementing the Trait
///
/// All methods in this crate are asynchronous. If you're planning on implementing this trait, use the `async_trait` crate.
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use dnd_lib::getter::DataProvider;
/// use dnd_lib::character::items::{Item, ItemType}
/// use dnd_lib::character::Background;
///
/// struct Retrievier;
///
/// #[async_trait]
/// impl DataProvider for Retrievier {
///     async fn get_item(&self, name: &str) -> Result<Item, CharacterDataError> {
///         Ok(Item {
///             name: name.to_string(),
///             description: None,
///             item_type: ItemType::Misc,
///         })
///     }
///
///     async fn get_background(&self, name: &str) -> Result<Background, CharacterDataError> {
///         // gets background from api
///     }
///}
///     
///```
///
/// There's no guarentee that different implementations return exactly the same values. It is
/// requested, though, that the name field always matches the name passed to the getter. (Save for
/// capitalization differences.)
///
///
/// ## Using Different Sources
/// You can use differnts sources within the same implementation, e,g, using one api for spells and
/// another for items. If your implmentation has a public version of a raw `get_class` that is
/// defined as get_class_raw(&impl DataProvider, name: &str) -> Result<Class, CharacterDataError>, then another crate can pass a different DataProvider to it
/// in order to change where the class retrieves items from.
#[async_trait]
pub trait DataProvider<E: Error>: Send + Sync {
    async fn get_race(&self, name: &str) -> Result<Race, E>;
    async fn get_background(&self, name: &str) -> Result<Background, E>;
    async fn get_item(&self, name: &str) -> Result<Item, E>;
    async fn get_class(&self, name: &str) -> Result<Class, E>;
    async fn get_spell(&self, name: &str) -> Result<Spell, E>;
}

