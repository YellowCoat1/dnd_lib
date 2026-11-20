use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use thiserror::Error;

use crate::{DataProvider, rules2014::{
    background::Background, class::Class, items::Item, race::Race, spells::Spell,
}};

mod backgrounds;
mod classes;
mod get_page;
mod items;
mod races;
mod spells;
mod names;

#[cfg(test)]
mod class_tests;

/// Gets D&D data from dnd5eapi.co's graphql api.
///
/// The following are availible from this api:
/// - classes: All except artificer
/// - items: Every basic item, no magic items
/// - backgrounds: only Acolyte
/// - races: Dragonborn, Dwarf, Elf, Gnome, Half-elf, Half-orc, Halfing, Human, Tiefling
///
/// ```rust
/// use dnd_lib::graphql::GraphqlProvider;
/// use dnd_lib::DataProvider;
/// use dnd_lib::rules2014::{items::Item, spells::Spell};
///
/// #[tokio::main]
/// async fn main() {
///     let provider = GraphqlProvider::new();
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
pub struct GraphqlProvider {
    equipment_cache: Mutex<(HashMap<String, Item>, bool)>,
}

/// An error that may occur with a [GraphqlProvider]
#[derive(Debug, Error)]
pub enum GraphqlError {
    #[error("Item Error: {0}")]
    Item(#[from] items::ItemError),
    #[error("Spell Error: {0}")]
    Spell(#[from] spells::SpellError),
    #[error("Race Error: {0}")]
    Race(#[from] races::RaceError),
    #[error("Background Error: {0}")]
    Background(#[from] backgrounds::BackgroundError),
    #[error("Class Error: {0}")]
    Class(#[from] classes::ClassError),
}

impl GraphqlProvider {
    pub fn new() -> Self {
        GraphqlProvider {
            equipment_cache: Mutex::new((HashMap::new(), false)),
        }
    }
}

impl Default for GraphqlProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::getter::DataProvider<GraphqlError> for GraphqlProvider {
    async fn get_race(&self, name: &str) -> Result<Race, GraphqlError> {
        races::get_race(name).await.map_err(|v| v.into())
    }
    async fn get_background(&self, name: &str) -> Result<Background, GraphqlError> {
        backgrounds::get_background(name)
            .await
            .map_err(|v| v.into())
    }
    async fn get_class(&self, name: &str) -> Result<Class, GraphqlError> {
        classes::get_class(name, self).await.map_err(|v| v.into())
    }
    async fn get_item(&self, name: &str) -> Result<Item, GraphqlError> {
        let get_database = || -> Result<_, _> {
            self.equipment_cache
                .lock()
                .map_err(|_| items::ItemError::DatabaseLockError)
        };

        // an edit of get_or_insert_with in order to support async stuffs
        let mut database_filled = true;
        if !get_database()?.1 {
            database_filled = false;
        }
        if !database_filled {
            let new_database = items::new_database().await?;
            get_database()?.0.extend(new_database);
        }

        get_database()?
            .0
            .get(&name.to_lowercase())
            .cloned()
            .ok_or(items::ItemError::DoesNotExist.into())
    }
    async fn get_spell(&self, name: &str) -> Result<Spell, GraphqlError> {
        spells::get_spell(name).await.map_err(|v| v.into())
    }
}

impl GraphqlProvider {
    /// The names of the spells available from the provider. Retrieved from the API.
    pub async fn spell_names(&self) -> Option<Vec<String>> {
        names::spell_names().await
    }
    /// The names of the classes available from the provider. Retrieved from the API.
    pub async fn class_names(&self) -> Option<Vec<String>> {
        names::class_names().await
    }
    /// The names of the races available from the provider. Retrieved from the API.
    pub async fn race_names(&self) -> Option<Vec<String>> {
        names::race_names().await
    }
    /// The names of the backgrounds available from the provider. Retrieved from the API.
    pub async fn background_names(&self) -> Option<Vec<String>> {
        names::background_names().await
    }
    /// The names of the items available from the provider. Retrieved from the API.
    pub async fn item_names(&self) -> Option<Vec<String>> {
        let _ = self.get_item(".").await; // dummy get to fill database if empty
        self.equipment_cache.lock().ok()?.0.keys().cloned().collect::<Vec<String>>().into()
    }
}
