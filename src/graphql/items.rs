use std::{collections::HashMap, sync::Mutex};
use graphql_client::{GraphQLQuery, Response};

use thiserror::Error;

use crate::rules2014::items::Item;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_query/schema.json"
    query_path = "graphql_query/query.graphql"
    response_derives = "Debug",
)]
struct EquipmentQuery;


#[derive(Debug, Error)]
pub enum ItemError {
    #[error("Failed to obtain item database!")]
    DatabaseLockError,
    #[error("Item does not exist.")]
    DoesNotExist,
}

pub fn get_item(name: &str, database: &Mutex<Option<HashMap<String, Item>>>) -> Result<Item, ItemError> {
    let mut database_locked = database.lock().map_err(|_| ItemError::DatabaseLockError)?;

    let v = database_locked.get_or_insert_with(new_database);
    v.get(name).cloned().ok_or(ItemError::DoesNotExist)
}

pub fn new_database() -> HashMap<String, Item> {
    let database = HashMap::new();
    database
}
