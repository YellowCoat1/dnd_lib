//! Traits and errors for retrieving D&D Character data.
use async_trait::async_trait;
use thiserror::Error;

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
pub trait DataProvider: Send + Sync {
    async fn get_race(&self, name: &str) -> Result<Race, CharacterDataError>;
    async fn get_background(&self, name: &str) -> Result<Background, CharacterDataError>;
    async fn get_item(&self, name: &str) -> Result<Item, CharacterDataError>;
    async fn get_class(&self, name: &str) -> Result<Class, CharacterDataError>;
    async fn get_spell(&self, name: &str) -> Result<Spell, CharacterDataError>;
}

/// Errors that can occur when retrieving or parsing character data
#[derive(Debug, Error)]
pub enum CharacterDataError {
    // Could not successfully connect to the api
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),
    // Could not properly deserialize the returned data
    #[error("failed to parse: {0}")]
    Parse(#[from] serde_json::Error),

    /// The api didn't have a required field
    #[error("Value not found: expected {val_type} named {name}")]
    NotFound {
        val_type: &'static str,
        name: String,
    },

    /// The api returned a field of an unexpected type
    #[error("type mismatch for field {field}: expected {expected}, got {found}")]
    TypeMismatch {
        field: String,
        expected: &'static str,
        found: String,
    },
}

impl CharacterDataError {
    /// Adds context by prefixing the `ValueMismatch` message.
    pub fn prepend(self, s: &str) -> CharacterDataError {
        match self {
            CharacterDataError::NotFound { val_type, name } => {
                let mut s = s.to_string();
                s.push_str(&name);
                CharacterDataError::NotFound { val_type, name: s }
            }
            CharacterDataError::TypeMismatch {
                field,
                expected,
                found,
            } => {
                let mut s = s.to_string();
                s.push_str(&field);
                CharacterDataError::TypeMismatch {
                    field: s,
                    expected,
                    found,
                }
            }
            o => o,
        }
    }

    /// Adds trailing context to a `ValueMismatch` message.
    pub fn append(mut self, s: &str) -> CharacterDataError {
        match &mut self {
            CharacterDataError::NotFound { name, .. } => {
                name.push_str(s);
            }
            CharacterDataError::TypeMismatch { field, .. } => {
                field.push_str(s);
            }
            _ => (),
        }

        self
    }

    /// Constructs a `ValueMismatch` with the given string.
    pub fn mismatch(field: &str, expected: &'static str, found: &str) -> CharacterDataError {
        CharacterDataError::TypeMismatch {
            field: field.to_string(),
            expected,
            found: found.to_string(),
        }
    }

    /// Constructs a `NotFound` with the given type and name.
    pub fn not_found(val_type: &'static str, name: &str) -> CharacterDataError {
        CharacterDataError::NotFound {
            val_type,
            name: name.to_string(),
        }
    }
}
