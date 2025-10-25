//! Gets data from dnd5eapi.co and converts it to the crate's datastructures.
//!
//! ```
//! use dnd_lib::get::get_spell;
//! #[tokio::main]
//! async fn main() {
//!     let blur = get_spell("blur").await.unwrap();
//!     assert_eq!(blur.name, "Blur");
//!     assert_eq!(blur.duration, "Up to 1 minute");
//! }
//! ```
mod get_page;
mod json_tools;
mod item;
mod spell;
mod feature;
mod background;
mod subrace;
mod race;
mod subclass;
mod class;


pub use feature::{get_feature, get_feature_from_trait};
pub use background::get_background;
pub use race::get_race;
pub use class::get_class;
pub use json_tools::ValueError;
pub use item::get_item;
pub use spell::get_spell;

#[cfg(test)]
#[cfg(feature = "network-intensive-tests")]
mod class_tests;
#[cfg(test)]
mod race_tests;
