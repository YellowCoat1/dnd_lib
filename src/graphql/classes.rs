use crate::{graphql::GraphqlProvider, rules2014::class::Class};
use graphql_client::{GraphQLQuery, Response};
use heck::ToKebabCase;
use thiserror::Error;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_query/schema.json"
    query_path = "graphql_query/query.graphql"
    response_derives = "Debug",
)]
pub struct ClassQuery;

#[derive(Debug, Error)]
pub enum ClassError {
    #[error("Class {0} does not exist.")]
    DoesNotExist(String),
    #[error("Failed to contact API.")]
    ConnectionError,
    #[error("Failed to parse {0} from item {0}")]
    ParseError(String, String),
}

pub async fn get_class(name: &str, provider: &GraphqlProvider) -> Result<Class, ClassError> {
    let response_data = request_class_page(name)
        .await
        .ok_or(ClassError::ConnectionError)?
        .class
        .ok_or(ClassError::DoesNotExist(name.to_string()))?;

    todo!()
}

async fn request_class_page(name: &str) -> Option<class_query::ResponseData> {
    //get data
    let vars = class_query::Variables {
        index: name.to_kebab_case(),
    };
    let request_body = ClassQuery::build_query(vars);

    let client = reqwest::Client::new();
    let res = client
        .post("https://www.dnd5eapi.co/graphql")
        .json(&request_body)
        .send()
        .await
        .ok()?;
    let response_body: Response<class_query::ResponseData> = res.json().await.ok()?;
    response_body.data
}
