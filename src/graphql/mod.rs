use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use thiserror::Error;

use crate::rules2014::{background::Background, class::Class, items::Item, spells::Spell, Race};

mod items;
use items::{ItemError, ItemErrorVariant};

pub struct GraphqlProvider {
    equipment_cache: Mutex<(HashMap<String, Item>, bool)>,
}

#[derive(Debug, Error)]
pub enum GraphqlError {
    #[error("Item Error: {0}")]
    Item(#[from] items::ItemError),
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
        todo!()
    }
    async fn get_background(&self, name: &str) -> Result<Background, GraphqlError> {
        todo!()
    }
    async fn get_class(&self, name: &str) -> Result<Class, GraphqlError> {
        todo!()
    }
    async fn get_item(&self, name: &str) -> Result<Item, GraphqlError> {
        let get_database = || -> Result<_, _> {
            self.equipment_cache
                .lock()
                .map_err(|_| ItemError::new(ItemErrorVariant::DatabaseLockError))
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
            .get(name)
            .cloned()
            .ok_or(ItemError::new(ItemErrorVariant::DoesNotExist).into())
    }
    async fn get_spell(&self, name: &str) -> Result<Spell, GraphqlError> {
        todo!()
    }
}
