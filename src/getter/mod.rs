//! General things for getting dnd data from an api
use thiserror::Error;
use async_trait::async_trait;

// A trait for structs that get d&d data from an api
#[async_trait]
pub trait DataProvider: Send + Sync {
    async fn get_race(&self, name: &str) -> Result<crate::character::Race, CharacterDataError>;
    async fn get_background(&self, name: &str) -> Result<crate::character::Background, CharacterDataError>;
    async fn get_item(&self, name: &str) -> Result<crate::character::items::Item, CharacterDataError>;
    async fn get_class(&self, name: &str) -> Result<crate::character::class::Class, CharacterDataError>;
    async fn get_spell(&self, name: &str) -> Result<crate::character::spells::Spell, CharacterDataError>;
}

/// A regular error from getting something from the api.
///
/// In the case of a CouldntGet, either you're offline, put in the wrong name, or really just any
/// regular error that would stop you from retrieving from the api.
/// 
/// If it's a ValueMismatch, it's reccomended that you contact the developer. They mean that there
/// was an error parsing from the api, which shouldn't happen regularly.
#[derive(Debug, Error)]
pub enum CharacterDataError {
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("failed to parse: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("unexpected value: {0}")]
    ValueMismatch(String),
}

impl CharacterDataError {
    /// Prepends a value to the error message.
    /// used for something a bit like a backtrace.
    pub fn prepend(self, s: &str) -> CharacterDataError {
        match self {
            CharacterDataError::ValueMismatch(t) => {
                let mut new_string = s.to_string();
                new_string.push_str(&t);
                CharacterDataError::ValueMismatch(new_string)
            }
            o => o
        }
    }

    pub fn append(mut self, s: &str) -> CharacterDataError {
        if let CharacterDataError::ValueMismatch(v) = &mut self {
            v.push_str(s);
        }
        self
    }

    pub fn new(s: &str) -> CharacterDataError {
        CharacterDataError::ValueMismatch(s.to_string())
    }
}
