use std::str::FromStr;

use crate::{
    graphql::{classes::class_query::ClassQueryClass, items::ItemError, GraphqlProvider},
    rules2014::{
        class::{Class, ClassBuilder, ClassBuilderError, TrackedField},
        stats::{EquipmentProficiencies, Saves, StatType},
    },
};
use graphql_client::{GraphQLQuery, Response};
use heck::ToKebabCase;
use thiserror::Error;

mod class_specific;
mod equipment;
mod features;
mod skills;
mod spellcasting;
mod subclasses;

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
    #[error("Item Retrieval Error: {0}")]
    ItemRetrieval(#[from] ItemError),
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

    let features = features::features(&response_data)?;

    let equipment = equipment::equipment(&response_data, provider).await?;
    let (equipment_proficiencies, saving_throw_proficiencies) =
        process_proficiencies(&response_data)?;

    let multiclass_prerequisites = process_multiclassing_requirements(&response_data)?;
    let multiclass_or = name.to_lowercase().as_str() == "fighter";
    let subclasses = subclasses::process_subclasses(&response_data)?;
    let spells = spellcasting::process_spellcasting(&response_data)?;
    let (skills_num, skills_choices) = skills::process_skills(&response_data)?;
    let class_specific = class_specific::process_class_specific(&response_data)?;

    let tracked_fields = process_tracked_fields(&response_data)
        .map(|v| vec![v])
        .unwrap_or(vec![]);
    ClassBuilder::new()
        .name(name.clone())
        .set_hit_die(hit_die)
        .add_equipment_proficiencies(equipment_proficiencies)
        .saving_throw_proficiencies(saving_throw_proficiencies)
        .add_multiclassing_prerequisites(multiclass_prerequisites)
        .set_multiclassing_prerequisites_or(multiclass_or)
        .add_beginning_items(equipment)
        .set_features(features)
        .add_subclasses(subclasses)
        .set_spellcasting(spells)
        .set_skill_proficiency_choices(skills_num, skills_choices)
        .add_class_specific_fields(class_specific)
        .add_tracked_fields(tracked_fields)
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

fn process_proficiencies(
    response_data: &ClassQueryClass,
) -> Result<(EquipmentProficiencies, Saves), ClassError> {
    let proficiency_strings =
        response_data
            .proficiencies
            .as_ref()
            .ok_or(ClassError::parse_error(
                "proficiencies",
                &response_data.name,
            ))?;
    let mut equipment = EquipmentProficiencies::default();
    let mut saves = Saves::default();
    let name = &response_data.name;

    for proficiency in proficiency_strings {
        match proficiency.name.to_lowercase().as_str() {
            "heavy armor" => equipment.heavy_armor = true,
            "light armor" => equipment.light_armor = true,
            "martial weapons" => equipment.martial_weapons = true,
            "medium armor" => equipment.medium_armor = true,
            "shields" => equipment.shields = true,
            "simple weapons" => equipment.simple_weapons = true,
            proficiency if proficiency.starts_with("saving throw: ") => match proficiency.get(14..)
            {
                Some("cha") => saves.charisma = true,
                Some("con") => saves.constitution = true,
                Some("dex") => saves.dexterity = true,
                Some("int") => saves.intelligence = true,
                Some("str") => saves.strength = true,
                Some("wis") => saves.wisdom = true,
                Some(o) => {
                    return Err(ClassError::parse_error(
                        format!("unrecognized stat proficiency {o}").as_str(),
                        name,
                    ))
                }
                None => {
                    return Err(ClassError::parse_error(
                        "saving throw name less than 14 chars",
                        name,
                    ))
                }
            },
            "crossbows, light" => {
                equipment.other.insert("light crossbows".to_string());
            }
            "crossbows, heavy" => {
                equipment.other.insert("heavy crossbows".to_string());
            }
            o => {
                equipment.other.insert(o.to_string());
            }
        }
    }

    Ok((equipment, saves))
}

fn process_multiclassing_requirements(
    response_data: &ClassQueryClass,
) -> Result<Vec<(StatType, usize)>, ClassError> {
    let name = &response_data.name;
    // a stand-in empty vec for if the real one doesn't exist
    let dummy_vec = vec![];
    response_data
        .multi_classing
        .as_ref()
        .ok_or(ClassError::parse_error("multiclassing", name))?
        .prerequisites
        .as_ref()
        .unwrap_or(&dummy_vec)
        .iter()
        .map(|v| {
            v.ability_score
                .as_ref()
                .map(|a| (a.name.clone(), v.minimum_score))
                .ok_or(ClassError::parse_error("multiclassing ability score", name))
        })
        .map(|z| {
            z.and_then(|(v1, v2)| {
                StatType::from_str(&v1)
                    .map_err(|_| ClassError::parse_error("multiclass stattype parsing", name))
                    .and_then(|st| {
                        v2.try_into()
                            .map_err(|_| ClassError::parse_error("multiclass stattype value", name))
                            .map(|z| (st, z))
                    })
            })
        })
        .collect::<Result<Vec<_>, ClassError>>()
}

fn process_tracked_fields(response_data: &ClassQueryClass) -> Option<TrackedField> {
    match response_data.name.to_lowercase().as_str() {
        "barbarian" => Some(TrackedField {
            name: "Rage".to_string(),
            long_rest: true,
            short_rest: false,
            level_up: false,
            class_specific_max: Some("Rages".to_string()),
            hard_max: None,
        }),
        "druid" => Some(TrackedField {
            name: "Wildshape".to_string(),
            long_rest: true,
            short_rest: true,
            level_up: false,
            class_specific_max: None,
            hard_max: Some(2),
        }),
        "sorcerer" => Some(TrackedField {
            name: "Sorcery Points".to_string(),
            long_rest: true,
            short_rest: false,
            level_up: false,
            class_specific_max: Some("Sorcery Points".to_string()),
            hard_max: None,
        }),
        "monk" => Some(TrackedField {
            name: "Ki Points".to_string(),
            long_rest: true,
            short_rest: false,
            level_up: false,
            class_specific_max: Some("Ki Points".to_string()),
            hard_max: None,
        }),
        _ => None,
    }
}
