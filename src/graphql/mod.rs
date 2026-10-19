use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use thiserror::Error;

use crate::rules2014::{Race, background::Background, class::Class, items::Item, spells::Spell};

pub struct GraphqlProvider {
    equipment_cache: Mutex<Option<HashMap<String, Item>>>,
}

#[derive(Debug, Error)]
pub enum GraphqlError {}


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
