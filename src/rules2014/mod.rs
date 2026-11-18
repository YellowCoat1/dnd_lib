//! Data types and rules needed to build and manage a 2014 D&D 5e character.
//!
//! The module is organized around immutable source data ([`class::Class`],
//! [`background::Background`], and [`race::Race`]) and the mutable
//! [`player_character::Character`] assembled from that data. Builders are provided for
//! homebrew content, while [`features::PresentedOption`] represents choices that still need
//! to be made during character creation.
//!
//! Most public data types implement `Serialize` and `Deserialize`, making character data
//! convenient to save or send over an API.
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
