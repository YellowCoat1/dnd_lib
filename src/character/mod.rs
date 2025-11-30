//! A module that contains the needed rules and etc factors to make a character.
//!
//! Every struct here implements serde serialize and deserialize, allowing for easy sending or
//! saving.
pub mod background;
mod choice;
pub mod features;
pub mod items;
mod race;
pub mod spells;
pub mod stats;
pub use race::*;
mod character_etc;
pub mod class;
pub mod player_character;
mod character_builder;

#[cfg(test)]
mod character_tests;

#[cfg(test)]
mod stats_tests;
