//! The needed rules and etc factors to make a character in 5e 2014 D&D.
//!
//! Every struct here implements serde serialize and deserialize, allowing for easy sending or
//! saving.
pub mod background;
mod character_builder;
mod character_etc;
mod choice;
pub mod class;
pub mod features;
pub mod items;
pub mod player_character;
pub mod race;
pub mod spells;
pub mod stats;

#[cfg(test)]
mod background_tests;
#[cfg(test)]
mod character_tests;
#[cfg(test)]
mod choice_tests;
#[cfg(test)]
mod race_tests;
#[cfg(test)]
mod spell_tests;
#[cfg(test)]
mod stats_tests;
