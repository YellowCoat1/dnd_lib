//! Traits and errors for retrieving D&D Character data.
use thiserror::Error;
use async_trait::async_trait;

// A trait representing a source capable of retrieving D&D data, e.g. from an api.
#[async_trait]
pub trait DataProvider: Send + Sync {
    async fn get_race(&self, name: &str) -> Result<crate::character::Race, CharacterDataError>;
    async fn get_background(&self, name: &str) -> Result<crate::character::Background, CharacterDataError>;
    async fn get_item(&self, name: &str) -> Result<crate::character::items::Item, CharacterDataError>;
    async fn get_class(&self, name: &str) -> Result<crate::character::class::Class, CharacterDataError>;
    async fn get_spell(&self, name: &str) -> Result<crate::character::spells::Spell, CharacterDataError>;
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
            CharacterDataError::NotFound {val_type, name} => {
                let mut s = s.to_string();
                s.push_str(&name);
                CharacterDataError::NotFound {val_type, name: s}
            },
            CharacterDataError::TypeMismatch { field , expected, found } => {
                let mut s = s.to_string();
                s.push_str(&field);
                CharacterDataError::TypeMismatch { field: s, expected, found }
            }
            o => o,
        }
    }

    /// Adds trailing context to a `ValueMismatch` message.
    pub fn append(mut self, s: &str) -> CharacterDataError {
        match &mut self {
            CharacterDataError::NotFound {name, ..} => {
                name.push_str(s);
            }
            CharacterDataError::TypeMismatch {field, ..} => {
                field.push_str(s);
            }
            _ => (),
        }

        self
    }

    /// Constructs a `ValueMismatch` with the given string.
    pub fn mismatch(field: &str, expected: &'static str, found: &str) -> CharacterDataError {
        CharacterDataError::TypeMismatch { field: field.to_string(), expected, found: found.to_string() }
    }
    
    pub fn not_found(val_type: &'static str, name: &str) -> CharacterDataError {
        CharacterDataError::NotFound { val_type, name: name.to_string() }
    }
}
