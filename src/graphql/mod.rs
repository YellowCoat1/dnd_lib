use std::{collections::HashMap, sync::Mutex};
use std::fmt::Display;

use async_trait::async_trait;
use thiserror::Error;

use crate::rules2014::{Race, background::Background, class::Class, items::Item, spells::Spell};

mod items;

pub struct GraphqlProvider {
    equipment_cache: Mutex<Option<HashMap<String, Item>>>,
}

#[derive(Debug, Error)]
pub enum GraphqlError {
    Item(items::ItemError),
}
impl Display for GraphqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphqlError::Item(i) => i.fmt(f),
        }
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
        todo!()
    }
    async fn get_spell(&self, name: &str) -> Result<Spell, GraphqlError> {
        todo!()
    }
}
