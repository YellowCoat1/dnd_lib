use crate::{
    graphql::races::race_query::RaceQueryRace,
    rules2014::{
        features::{Feature, FeatureEffect, PresentedOption},
        stats::{Size, SkillType, StatType},
        Race, RaceBonus, RaceBuilder, Subrace, SubraceBuilder,
    },
};
use graphql_client::{GraphQLQuery, Response};
use heck::ToKebabCase;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RaceError {
    #[error("Race {0} does not exist.")]
    DoesNotExist(String),
    #[error("Failed to contact API.")]
    ConnectionError,
    #[error("Failed to parse {0} from requested race {1}")]
    ParseError(String, String),
}

impl RaceError {
    fn parse_error(message: &str, name: &str) -> RaceError {
        RaceError::ParseError(message.to_string(), name.to_string())
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_query/schema.json"
    query_path = "graphql_query/query.graphql"
    response_derives = "Debug",
)]
pub struct RaceQuery;

pub async fn get_race(name: &str) -> Result<Race, RaceError> {
    let response_data = request_spell_page(name)
        .await
        .ok_or(RaceError::ConnectionError)?
        .race
        .ok_or(RaceError::DoesNotExist(name.to_string()))?;

    let name = response_data.name.clone();
    let size = Size::try_from(response_data.size.as_str())
        .map_err(|_| RaceError::parse_error("Size", &name))?;
    let speed: usize = response_data
        .speed
        .try_into()
        .map_err(|_| RaceError::parse_error("Speed", &name))?;
    let ability_bonuses = process_ability_bonuses(&response_data)?;
    let ability_bonus_choices_num: usize = response_data
        .ability_bonus_options
        .as_ref()
        .and_then(|v| v.choose.try_into().ok())
        .unwrap_or(0);
    let ability_bonus_size = response_data
        .ability_bonus_options
        .as_ref()
        .and_then(|v| v.from.options.first().map(|v| v.bonus as isize))
        .unwrap_or(0);
    let ability_bonus_choices =
        vec![RaceBonus::Wildcard(ability_bonus_size); ability_bonus_choices_num];

    let traits = response_data
        .traits
        .as_ref()
        .ok_or(RaceError::parse_error("Traits", &name))?
        .iter()
        .map(|v| {
            PresentedOption::Base(Feature {
                name: v.name.clone(),
                description: v.desc.clone(),
                effects: trait_effects(&v.name),
            })
        });
    let languages = response_data
        .languages
        .as_ref()
        .iter()
        .flat_map(|v| v.iter())
        .map(|v| v.name.clone())
        .collect::<Vec<_>>();
    let wildcard_languages: usize = response_data
        .language_options
        .as_ref()
        .map(|v| v.choose)
        .unwrap_or(0)
        .try_into()
        .map_err(|_| RaceError::parse_error("Wildcard Language Number", &name))?;

    let subraces = race_subraces(&response_data)?;

    Ok(RaceBuilder::new(&name)
        .size(size)
        .add_subraces(subraces)
        .speed(speed)
        .add_ability_bonuses(ability_bonuses)
        .add_ability_bonuses(ability_bonus_choices)
        .add_traits(traits)
        .add_languages(languages)
        .add_wildcard_languages(wildcard_languages)
        .build())
}

fn trait_effects(trait_name: &str) -> Vec<FeatureEffect> {
    match trait_name {
        "Keen Senses" => vec![FeatureEffect::AddSkillProficiency(SkillType::Perception)],
        "Menacing" => vec![FeatureEffect::AddSkillProficiency(SkillType::Intimidation)],
        _ => vec![],
    }
}

fn race_subraces(response_data: &RaceQueryRace) -> Result<Vec<Subrace>, RaceError> {
    let name = &response_data.name;
    response_data
        .subraces
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .map(|subrace| {
            let subrace_name = subrace.name.clone();
            let subrace_traits = subrace
                .racial_traits
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .map(|v| {
                    PresentedOption::Base(Feature {
                        name: v.name.clone(),
                        description: v.desc.clone(),
                        effects: trait_effects(&v.name),
                    })
                })
                .collect::<Vec<_>>();
            let bonuses = subrace
                .ability_bonuses
                .iter()
                .map(|v| {
                    v.ability_score
                        .as_ref()
                        .map(|z| (&z.name, v.bonus))
                        .ok_or(RaceError::parse_error("Ability Score", name))
                })
                .map(|z| {
                    z.and_then(|(ability, bonus)| {
                        let stat_type = StatType::from_shorthand(ability)
                            .ok_or(RaceError::parse_error("Subrace Stat Type", name))?;
                        Ok((Some(stat_type), bonus as isize))
                    })
                })
                .collect::<Result<Vec<_>, RaceError>>()?;

            Ok(SubraceBuilder::new(&subrace_name)
                .description(subrace.desc.clone())
                .add_ability_bonuses(bonuses)
                .add_traits(subrace_traits)
                .build())
        })
        .collect::<Result<Vec<Subrace>, RaceError>>()
}

fn process_ability_bonuses(response_data: &RaceQueryRace) -> Result<Vec<RaceBonus>, RaceError> {
    let name = &response_data.name;
    response_data
        .ability_bonuses
        .iter()
        .map(|v| {
            v.ability_score
                .as_ref()
                .map(|z| (&z.name, v.bonus))
                .ok_or(RaceError::parse_error("Ability Score", name))
        })
        .map(|v| {
            v.and_then(|(ability, bonus_unparsed)| {
                let stat_type = StatType::from_shorthand(ability)
                    .ok_or(RaceError::parse_error("Ability Score Stat Type", name))?;
                let bonus: isize = bonus_unparsed as isize;
                Ok(RaceBonus::Specific(stat_type, bonus))
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

async fn request_spell_page(name: &str) -> Option<race_query::ResponseData> {
    //get data
    let vars = race_query::Variables {
        index: name.to_kebab_case(),
    };
    let request_body = RaceQuery::build_query(vars);

    let client = reqwest::Client::new();
    let res = client
        .post("https://www.dnd5eapi.co/graphql")
        .json(&request_body)
        .send()
        .await
        .ok()?;
    let response_body: Response<race_query::ResponseData> = res.json().await.ok()?;
    response_body.data
}

#[cfg(test)]
mod graphql_race_tests {
    use crate::rules2014::{features::FeatureEffect, stats::Size, stats::StatType, RaceBonus};
    #[tokio::test]
    async fn get_dragonborn() {
        let dragonborn = super::get_race("dragonborn")
            .await
            .expect("failed to get dragonborn!");
        assert_eq!(*dragonborn.size(), Size::Medium);
    }
    #[tokio::test]
    async fn get_elf() {
        let elf = super::get_race("elf").await.expect("failed to get elf!");
        assert_eq!(
            (elf.name(), elf.speed(), *elf.size()),
            ("Elf", 30, Size::Medium)
        );
        assert_eq!(
            elf.ability_bonuses().first().cloned(),
            Some(RaceBonus::Specific(StatType::Dexterity, 2))
        );
        assert_eq!(
            elf.languages().first().cloned(),
            Some(String::from("Common"))
        );

        let mut high_elf = elf
            .subraces()
            .first()
            .expect("Elf should have subraces!")
            .clone();

        assert_eq!(high_elf.name(), "High Elf");
        assert_eq!(
            high_elf.ability_bonuses().first().cloned(),
            Some((Some(StatType::Intelligence), 1))
        );

        assert_eq!(high_elf.ability_bonuses_unchosen(), vec![]);
        assert_eq!(&high_elf.description()[..14], "As a high elf,");
        assert_eq!(
            high_elf.trait_effects_mut(),
            Vec::<&mut FeatureEffect>::new()
        );
    }
}
