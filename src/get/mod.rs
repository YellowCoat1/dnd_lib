//! Gets data from dnd5eapi.co
//!
//! The main feature of this module is the [Dnd5eapigetter], which implements [DataProvider](crate::getter::DataProvider)
//! trait.

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

use std::collections::HashMap;
use std::sync::Mutex;
use async_trait::async_trait;

use feature::get_feature as get_feature_inner;
use background::get_background as get_background_inner;
use race::get_race as get_race_inner;
use class::get_class as get_class_inner;
use item::get_item as get_item_inner;
use spell::get_spell as get_spell_inner;

use crate::{character::{Background, class::Class, features::Feature, items::Item}, getter::CharacterDataError};

/// Gets D&D data from dnd5eapi.co
///
/// The following are availible from this api:
/// - classes: All except artificier
/// - items: Every basic item, no magic items
/// - backgrounds: only Acolyte
/// - races: Dragonborn, Dwarf, Elf, Gnome, Half-elf, Half-orc, Halfing, Human, Tiefling
///
/// ```rust 
/// use dnd_lib::get::Dnd5eapigetter;
/// use dnd_lib::getter::DataProvider;
/// use dnd_lib::character::{items::Item, spells::Spell};
///
/// #[tokio::main]
/// async fn main() {
///     let provider = Dnd5eapigetter::new();
/// 
///     let item: Item = provider.get_item("shortsword")
///         .await.expect("failed to get shortsword");
///     assert_eq!(item.name, "Shortsword");
///
///     let spell: Spell = provider.get_spell("fireball")
///         .await.expect("failed to get fireball");
///     assert_eq!(spell.name, "Fireball");
///     assert_eq!(spell.range, "150 feet");
/// }
/// ```
pub struct Dnd5eapigetter {
    item_cache: Mutex<HashMap<String, Item>>,
    class_cache: Mutex<HashMap<String, Class>>,
    background_cache: Mutex<HashMap<String, Background>>,
}

#[async_trait]
impl crate::getter::DataProvider for Dnd5eapigetter {
    async fn get_race(&self, name: &str) -> Result<crate::character::Race, crate::getter::CharacterDataError> {
        get_race_inner(name).await
    }
    async fn get_background(&self, name: &str) -> Result<crate::character::Background, crate::getter::CharacterDataError> {
        if let Some(cached) = self.background_cache.lock().unwrap().get(name) {
            return Ok(cached.clone())
        }
        let background = get_background_inner(name).await?;
        self.background_cache.lock().unwrap().insert(name.to_string(), background.clone());
        Ok(background)
    }
    async fn get_class(&self, name: &str) -> Result<crate::character::class::Class, crate::getter::CharacterDataError> {
        if let Some(cached) = self.class_cache.lock().unwrap().get(name) {
            return Ok(cached.clone())
        }
        let class = get_class_inner(name).await?;
        self.class_cache.lock().unwrap().insert(name.to_string(), class.clone());
        Ok(class)
    }
    async fn get_item(&self, name: &str) -> Result<crate::character::items::Item, crate::getter::CharacterDataError> {
        if let Some(cached) = self.item_cache.lock().unwrap().get(name) {
            return Ok(cached.clone())
        }
        let item = get_item_inner(name).await?;
        self.item_cache.lock().unwrap().insert(name.to_string(), item.clone());
        Ok(item)
    }
    async fn get_spell(&self, name: &str) -> Result<crate::character::spells::Spell, crate::getter::CharacterDataError> {
        get_spell_inner(name).await
    }
}

impl Dnd5eapigetter {
    pub fn new() -> Dnd5eapigetter {
        Dnd5eapigetter { 
            item_cache: Mutex::new(HashMap::new()),
            class_cache: Mutex::new(HashMap::new()),
            background_cache: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_feature(&self, name: &str) -> Result<Feature, CharacterDataError> {
        get_feature_inner(name).await
    }
}

impl Default for Dnd5eapigetter {
    fn default() -> Self {
        Dnd5eapigetter { 
            item_cache: Mutex::new(HashMap::new()),
            class_cache: Mutex::new(HashMap::new()), 
            background_cache:  Mutex::new(HashMap::new())
        }
    }
}


#[cfg(test)]
#[cfg(feature = "network-intensive-tests")]
mod class_tests;
#[cfg(test)]
mod race_tests;
