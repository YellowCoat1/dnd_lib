//! A library to assist in dnd 5e's systems.
//!
//! The main feature of this crate is the [Character](character::Character) struct. Most of the
//! other datastructures in this crate (found in the [character] module) are centered around it,
//! building the foundations for a complete D&D character.
//!
//! You'll most likely want to start by getting the basic structures ([class](character::class::Class), [background](character::background::Background), [race](character::race::Race), etc) from the [get] module, and using
//! that to build a character.

pub mod character;
pub mod get;
pub mod save;

#[cfg(test)]
mod comprehensive_tests;
