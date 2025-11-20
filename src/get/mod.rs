//! Gets data from dnd5eapi.co
//!
//! The main feature of this module is the [Dnd5eapigetter], which implements [DataProvider](crate::getter::DataProvider)
//! trait.

mod background;
mod class;
mod feature;
mod get_page;
mod item;
mod json_tools;
mod race;
mod spell;
mod subclass;
mod subrace;

pub mod raw_getters {
    //! Raw getters for dnd5eapi.co data. 
    //!
    //! These are the individual functions used by the
    //! [Dnd5eapigetter](super::Dnd5eapigetter) to get data. Regular users should prefer using the [Dnd5eapigetter](super::Dnd5eapigetter)
    //! directly.
    //!
    //! These functions are exposed primarily for users who want to implement their own
    //! [DataProvider](crate::DataProvider) trait, using dnd5eapi.co as one backend.

    pub use super::feature::get_feature as get_feature_raw;
    pub use super::background::get_background as get_background_raw;
    pub use super::class::get_class as get_class_raw;
    pub use super::item::get_item as get_item_raw;
    pub use super::race::get_race as get_race_raw;
    pub use super::spell::get_spell as get_spell_raw;
}

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use background::get_background as get_background_inner;
use class::get_class as get_class_inner;
use feature::get_feature as get_feature_inner;
use item::get_item as get_item_inner;
use race::get_race as get_race_inner;
use spell::get_spell as get_spell_inner;

use crate::{
    character::{class::Class, features::Feature, items::Item, Background},
    getter::CharacterDataError,
};

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
/// Do note that this getter can be quite slow, as it needs to make multiple network requests to
/// get all the data.
/// Caching is implemented for items, classes, and backgrounds to help with this.
pub struct Dnd5eapigetter {
    item_cache: Mutex<HashMap<String, Item>>,
    class_cache: Mutex<HashMap<String, Class>>,
    background_cache: Mutex<HashMap<String, Background>>,
}

#[async_trait]
impl crate::getter::DataProvider for Dnd5eapigetter {
    async fn get_race(
        &self,
        name: &str,
    ) -> Result<crate::character::Race, crate::getter::CharacterDataError> {
        let mut c = get_race_inner(name).await?;
        capitalize(&mut c.name);
        Ok(c)
    }
    async fn get_background(
        &self,
        name: &str,
    ) -> Result<crate::character::Background, crate::getter::CharacterDataError> {
        if let Some(cached) = self.background_cache.lock().unwrap().get(name) {
            return Ok(cached.clone());
        }
        let mut background = get_background_inner(self, name).await?;
        capitalize(&mut background.name);
        self.background_cache
            .lock()
            .unwrap()
            .insert(name.to_string(), background.clone());
        Ok(background)
    }
    async fn get_class(
        &self,
        name: &str,
    ) -> Result<crate::character::class::Class, crate::getter::CharacterDataError> {
        if let Some(cached) = self.class_cache.lock().unwrap().get(name) {
            return Ok(cached.clone());
        }
        let mut class = get_class_inner(self, name).await?;
        capitalize(&mut class.name);
        self.class_cache
            .lock()
            .unwrap()
            .insert(name.to_string(), class.clone());
        Ok(class)
    }
    async fn get_item(
        &self,
        name: &str,
    ) -> Result<crate::character::items::Item, crate::getter::CharacterDataError> {
        if let Some(cached) = self.item_cache.lock().unwrap().get(name) {
            return Ok(cached.clone());
        }
        let mut item = get_item_inner(name).await?;
        capitalize(&mut item.name);
        self.item_cache
            .lock()
            .unwrap()
            .insert(name.to_string(), item.clone());
        Ok(item)
    }
    async fn get_spell(
        &self,
        name: &str,
    ) -> Result<crate::character::spells::Spell, crate::getter::CharacterDataError> {
        let mut s = get_spell_inner(name).await?;
        capitalize(&mut s.name);
        Ok(s)
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
            background_cache: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "network-intensive-tests")]
mod class_tests;
#[cfg(test)]
mod race_tests;

// Capitalize the first character of a string
fn capitalize(s: &mut String) {
    let c = s.remove(0);
    s.insert(0, c.to_ascii_uppercase());
}

#[cfg(test)]
mod test {
    use crate::character::features::PresentedOption;
    use crate::character::stats::SkillType;
    use crate::getter::DataProvider;
    use crate::provider;

    #[tokio::test]
    async fn get_acolyte() {
        let provider = provider();
        let acolyte = provider
            .get_background("acolyte")
            .await
            .expect("failed to get acolyte!");
        let insight = acolyte
            .proficiencies
            .first()
            .expect("acolyte should have proficiencies!");
        assert_eq!(*insight, PresentedOption::Base(SkillType::Insight));
        let hero = acolyte
            .personality_traits
            .first()
            .expect("acolyte should have personality traits!");
        assert_eq!(*hero, String::from("I idolize a particular hero of my faith, and constantly refer to that person's deeds and example."));
        let tradition = acolyte.ideals.first().expect("acolyte should have ideals!");
        assert_eq!(*tradition, String::from("Tradition. The ancient traditions of worship and sacrifice must be preserved and upheld."));
    }
}
