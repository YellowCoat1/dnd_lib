
use graphql_client::GraphQLQuery;

use crate::graphql::get_page::request_graphql_page;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_query/schema.json"
    query_path = "graphql_query/query.graphql"
    response_derives = "Debug",
)]
struct Names;


pub async fn race_names() -> Option<Vec<String>> {
    let vars = names::Variables;
    let names = request_graphql_page::<Names>(vars)
        .await?
        .races
        .into_iter()
        .map(|v| v.name)
        .collect();
    Some(names)
}

pub async fn background_names() -> Option<Vec<String>> {
    let vars = names::Variables;
    let names = request_graphql_page::<Names>(vars)
        .await?
        .backgrounds
        .into_iter()
        .map(|v| v.name)
        .collect();
    Some(names)
}

pub async fn class_names() -> Option<Vec<String>> {
    let vars = names::Variables;
    let names = request_graphql_page::<Names>(vars)
        .await?
        .classes
        .into_iter()
        .map(|v| v.name)
        .collect();
    Some(names)
}

pub async fn spell_names() -> Option<Vec<String>> {
    let vars = names::Variables;
    let names = request_graphql_page::<Names>(vars)
        .await?
        .spells
        .into_iter()
        .map(|v| v.name)
        .collect();
    Some(names)
}
