//! A module that contains the needed rules and etc factors to make a character.
//!
//! Every struct here implements serde serialize and deserialize, allowing for easy sending or
//! saving.
pub mod background;
mod choice;
pub mod features;
pub mod items;
pub mod spells;
pub mod stats;
mod race;
pub use race::*;
mod character_etc;
pub mod class;
pub use character_etc::{CharacterDescriptors, CharacterStory};
mod player_character;
pub use player_character::{Castable, Character, SpeccedClass};
mod character_builder;
pub use character_builder::CharacterBuilder;

#[cfg(test)]
mod character_tests;

#[cfg(test)]
mod stats_tests;
