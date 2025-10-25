//! A module that contains the needed rules and etc factors to make a character.
//!
//! Every struct here implements serde serialize and deserialize, allowing for easy sending or
//! saving.
pub mod stats;
mod choice;
pub mod features;
pub mod items;
pub mod spells;
mod background;
pub use background::Background;
mod race;
pub use race::{Race, Subrace};
pub mod class;
mod character;
pub use character::{Character, SpeccedClass};

#[cfg(test)]
mod character_tests;

#[cfg(test)]
mod stats_tests;
