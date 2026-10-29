use graphql_client::{GraphQLQuery, Response};
use heck::ToKebabCase;
use thiserror::Error;

use crate::rules2014::{
    background::{Background, BackgroundBuildError, BackgroundBuilder, LanguageOption},
    features::{Feature, PresentedOption},
    items::{Item, ItemType},
    stats::SkillType,
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_query/schema.json"
    query_path = "graphql_query/query.graphql"
    response_derives = "Debug",
)]
pub struct BackgroundQuery;

#[derive(Debug, Error)]
pub enum BackgroundError {
    #[error("Background {0} does not exist.")]
    DoesNotExist(String),
    #[error("Failed to contact API.")]
    ConnectionError,
    #[error("Failed to parse {0} from requested spell {1}")]
    ParseError(String, String),
}

impl BackgroundError {
    fn parse_error(msg: &str, name: &str) -> BackgroundError {
        BackgroundError::ParseError(msg.to_string(), name.to_string())
    }
}

pub async fn get_background(name: &str) -> Result<Background, BackgroundError> {
    let response_data = request_background_page(name)
        .await
        .ok_or(BackgroundError::ConnectionError)?
        .background
        .ok_or(BackgroundError::DoesNotExist(name.to_string()))?;

    let name = response_data.name.clone();
    let bonds = response_data
        .bonds
        .as_ref()
        .ok_or(BackgroundError::parse_error("bonds", &name))?
        .from
        .options
        .iter()
        .map(|v| v.string.clone());
    let ideals = response_data
        .ideals
        .as_ref()
        .ok_or(BackgroundError::parse_error("ideals", &name))?
        .from
        .options
        .iter()
        .map(|v| v.desc.clone());
    let flaws = response_data
        .flaws
        .as_ref()
        .ok_or(BackgroundError::parse_error("flaws", &name))?
        .from
        .options
        .iter()
        .map(|v| v.string.clone());
    let personality_traits = response_data
        .personality_traits
        .as_ref()
        .ok_or(BackgroundError::parse_error("flaws", &name))?
        .from
        .options
        .iter()
        .map(|v| v.string.clone());

    let skills = response_data
        .starting_proficiencies
        .iter()
        .map(|v| {
            v.name
                .get(7..)
                .ok_or(BackgroundError::parse_error("Skills", &name))
        })
        .map(|v| {
            v.and_then(|y| {
                SkillType::from_name(y).ok_or(BackgroundError::parse_error("Skills Parsing", &name))
            })
        })
        .map(|v| v.map(PresentedOption::Base))
        .collect::<Result<Vec<_>, _>>()?;

    let feature_name = &response_data.feature.name;
    let description = response_data.feature.desc.clone();

    let feature = Feature {
        name: feature_name.clone(),
        description,
        effects: vec![],
    };
    let equipment = bg_equipment(&response_data)?;
    let languages = bg_languages(&response_data)?;
    BackgroundBuilder::new(&name)
        .add_proficiencies(skills)
        .add_equipment_set(equipment)
        .add_feature(feature)
        .add_personality_traits(personality_traits)
        .add_flaws(flaws)
        .add_bonds(bonds)
        .add_ideals(ideals)
        .add_language_options(languages)
        .build()
        .map_err(|o| {
            let err_str = match o {
                BackgroundBuildError::NotEnoughPersonalityTraits => {
                    "Building Insufficient Personality Traits"
                }
                BackgroundBuildError::EmptyBonds => "Building Empty Bonds",
                BackgroundBuildError::EmptyProficiencies => "Building Empty Proficiencies",
                BackgroundBuildError::EmptyFlaws => "Building Empty Flaws",
                BackgroundBuildError::EmptyIdeals => "Building Empty Ideals",
            };
            BackgroundError::parse_error(err_str, &name)
        })
}

pub fn bg_equipment(
    response_data: &background_query::BackgroundQueryBackground,
) -> Result<Vec<(Item, usize)>, BackgroundError> {
    let bg_name = &response_data.name;
    response_data
        .starting_equipment
        .iter()
        .map(|v| (v.equipment.name.clone(), v.quantity))
        .map(|(name, n)| {
            (
                Item {
                    name,
                    description: None,
                    item_type: ItemType::Misc,
                    features: vec![],
                },
                n,
            )
        })
        .map(|(item, count)| {
            count
                .try_into()
                .map_err(|_| BackgroundError::parse_error("item count", bg_name))
                .map(|c| (item, c))
        })
        .collect::<Result<Vec<(Item, usize)>, BackgroundError>>()
}

pub fn bg_languages(
    response_data: &background_query::BackgroundQueryBackground,
) -> Result<Vec<LanguageOption>, BackgroundError> {
    let name = &response_data.name;
    let language_info = response_data
        .language_options
        .as_ref()
        .ok_or(BackgroundError::parse_error("languages", name))?;
    let langs = language_info
        .from
        .options
        .iter()
        .map(|v| v.item.name.to_lowercase())
        .collect::<Vec<_>>();
    let mut lang_choice = LanguageOption::new_named_choice(langs);
    if let LanguageOption::NamedChoice(ref v) = lang_choice {
        if v.len() >= 8 {
            lang_choice = LanguageOption::UnnamedChoice;
        }
    }
    let count: usize = language_info
        .choose
        .try_into()
        .map_err(|_| BackgroundError::parse_error("Language count", name))?;
    Ok(vec![lang_choice; count])
}

async fn request_background_page(name: &str) -> Option<background_query::ResponseData> {
    //get data
    let vars = background_query::Variables {
        index: name.to_kebab_case(),
    };
    let request_body = BackgroundQuery::build_query(vars);

    let client = reqwest::Client::new();
    let res = client
        .post("https://www.dnd5eapi.co/graphql")
        .json(&request_body)
        .send()
        .await
        .ok()?;
    let response_body: Response<background_query::ResponseData> = res.json().await.ok()?;
    response_body.data
}

#[cfg(test)]
mod bg_tests {
    use super::get_background;
    #[tokio::test]
    async fn bg_test() {
        let acolyte = get_background("acolyte").await.unwrap();
        assert_eq!(acolyte.name(), "Acolyte");
    }
}
