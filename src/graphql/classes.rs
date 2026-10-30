use crate::{
    graphql::GraphqlProvider,
    rules2014::class::{Class, ClassBuilder, ClassBuilderError},
};
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

impl ClassError {
    fn parse_error(msg: &str, name: &str) -> ClassError {
        ClassError::ParseError(msg.to_string(), name.to_string())
    }
}

pub async fn get_class(name: &str, provider: &GraphqlProvider) -> Result<Class, ClassError> {
    let response_data = request_class_page(name)
        .await
        .ok_or(ClassError::ConnectionError)?
        .class
        .ok_or(ClassError::DoesNotExist(name.to_string()))?;

    let name = &response_data.name;
    let hit_die = response_data
        .hit_die
        .try_into()
        .map_err(|_| ClassError::parse_error("hit die", name))?;

    let features = todo!();
    let equipment = todo!();
    let multiclass_prerequisites = todo!();
    let multiclass_or = todo!();
    let saving_throw_proficiencies = todo!();
    let equipment_proficiencies = todo!();
    let subclasses = todo!();
    let spells = todo!();
    let skills_num = todo!();
    let skills_choices = todo!();

    ClassBuilder::new()
        .name(name.clone())
        .set_hit_die(hit_die)
        .add_equipment_proficiencies(equipment_proficiencies)
        .saving_throw_proficiencies(saving_throw_proficiencies)
        //.add_multiclassing_prerequisites(multiclass_prerequisites)
        .set_multiclassing_prerequisites_or(multiclass_or)
        //.add_beginning_items(equipment)
        .set_features(features)
        //.add_subclasses(subclasses)
        .set_spellcasting(spells)
        .set_skill_proficiency_choices(skills_num, skills_choices)
        .build()
        .map_err(|v| match v {
            ClassBuilderError::NameMissing => ClassError::parse_error("missing name", name),
            ClassBuilderError::HitDieMissing => ClassError::parse_error("hit die missing", name),
            ClassBuilderError::FeaturesMissing => ClassError::parse_error("features missing", name),
        })
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
