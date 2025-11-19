//! A library to assist in creating and managing D&D 5e characters.
//!
//! The main feature of this crate is the [Character] struct. Most of the
//! other datastructures in this crate (found in the [character] module) are centered around it,
//! building the foundations for a complete D&D character.
//!
//! ```
//! # #[cfg(feature = "dnd5eapi")] {
//! #[tokio::main]
//! async fn main() {
//!     use rand::Rng;
//!     use dnd_lib::prelude::*;
//!     let mut rng = rand::thread_rng();
//!
//!     // first, we construct the api getter.
//!     let provider = Dnd5eapigetter::new();
//!
//!     // then, we get all the things we need to create a character.
//!     let rogue = provider.get_class("rogue").await.unwrap();
//!     let human = provider.get_race("human").await.unwrap();
//!     let acolyte = provider.get_background("acolyte").await.unwrap();
//!
//!     // this is john. John is a human rogue.
//!     let mut john = CharacterBuilder::new("John")
//!         .race(&human)
//!         .background(&acolyte)
//!         .class(&rogue)
//!         .stats(Stats::default())
//!         .build().unwrap();
//!     
//!     // john sees an upcoming fight, and equips his dagger.
//!     john.items[3].2 = true;
//!
//!     // Uh-Oh! John is about to get hit! What's his AC?
//!     let ac = john.ac();
//!     // looks like it was too small. John gets hit with 3 damage.
//!     john.damage(3);
//!     // Now it's John's turn. He readies his dagger.
//!     let dagger_attack = &john.weapon_actions()[0];
//!     // John tries to attack...
//!     let attack_roll = rng.random_range(1..=20) as isize + dagger_attack.attack_bonus;
//!     // And it hits!
//!     let damage_roll = dagger_attack.damage_roll;
//!     // It does enough damage to kill the monster immediately!
//!
//!     // With the xp from that fight, john levels up.
//!     john.level_up(&rogue);
//!
//!     // Afterwards, john is smited from reality for the sin of existance.
//!     drop(john);
//! }
//! # }
//! ```
//!
//! This crate stores different choices for a character as different structs. For example, a
//! [character::class::Class] would be a Wizard, or a Figter, or a Monk. If you wanted to store every D&D class a
//! character could take, you'd need a `Vec<Class>`. These rules must be parsed from an api and constructed.
//!
//! This is what [get::Dnd5eapigetter] is for. You first get the required rules (class, background,
//! race) from the api, then you build a character with that.
//!
//! ## Feature flags
//!
//! - `integration` Specifically for testing. Enables all tests.
//! - `dnd5eapi` - *(enabled by default)* Enables retrieving through the dnd5eapi.co api.

pub mod character;
#[cfg(feature = "dnd5eapi")]
pub mod get;
mod getter;
pub mod save;

// re-exports
pub use getter::*;
pub use character::Character;
pub use character::CharacterBuilder;

#[cfg_attr(not(test), allow(dead_code))]
#[cfg(feature = "dnd5eapi")]
use std::sync::{Arc, OnceLock};
#[cfg(feature = "dnd5eapi")]
#[cfg_attr(not(test), allow(dead_code))]
static PROVIDER: OnceLock<Arc<get::Dnd5eapigetter>> = OnceLock::new();

// Global api getter for all tests, in order for a shared cache
#[cfg(test)]
#[cfg(feature = "dnd5eapi")]
pub(crate) fn provider() -> Arc<get::Dnd5eapigetter> {
    PROVIDER
        .get_or_init(|| Arc::new(get::Dnd5eapigetter::new()))
        .clone()
}

pub mod prelude {
    #[cfg(feature = "dnd5eapi")]
    pub use crate::get::Dnd5eapigetter;
    pub use crate::{
        character::class::Class,
        character::stats::Stats,
        character::{Background, Character, CharacterBuilder, Race},
        getter::{CharacterDataError, DataProvider},
    };
}
