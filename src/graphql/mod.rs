use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use thiserror::Error;

use crate::rules2014::{background::Background, class::Class, items::Item, spells::Spell, race::Race};

mod backgrounds;
mod classes;
mod items;
mod races;
mod spells;

#[cfg(test)]
mod class_tests;

/// Gets D&D data from dnd5eapi.co's graphql api.
///
/// The following are availible from this api:
/// - classes: All except artificier
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
