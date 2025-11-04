//! A library to assist in creating and managing D&D 5e characters.
//!
//! The main feature of this crate is the [Character](character::Character) struct. Most of the
//! other datastructures in this crate (found in the [character] module) are centered around it,
//! building the foundations for a complete D&D character.
//!
//! You'll most likely want to start by getting the basic structures ([class](character::class::Class), [background](character::Background), [race](character::Race), etc) from the [get] module, and using
//! that to build a character.
//!
//! ```
//! #[tokio::main]
//! async fn main() {
//!     extern crate rand;
//!     use rand::Rng;
//!     use dnd_lib::{getter::DataProvider, get::Dnd5eapigetter};
//!     use dnd_lib::character::{stats::Stats, Character};
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
//!     let mut john = Character::new(String::from("john"), &rogue, &acolyte, &human, Stats::default());
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
//! ```
//!


pub mod character;
pub mod getter;
pub mod get;
pub mod save;

#[cfg(test)]
mod comprehensive_tests;

#[cfg_attr(not(test), allow(dead_code))]
use std::sync::{Arc, OnceLock};
#[cfg_attr(not(test), allow(dead_code))]
static PROVIDER: OnceLock<Arc<get::Dnd5eapigetter>> = OnceLock::new();

#[cfg(test)]
pub(crate) fn provider() -> Arc<get::Dnd5eapigetter> {
    PROVIDER.get_or_init(|| Arc::new(get::Dnd5eapigetter::new())).clone()
}


