use std::str::FromStr;

use crate::rules2014::{
    items::{DamageRoll, DamageType},
    spells::{School, Spell},
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
pub struct SpellQuery;

#[derive(Debug, Error)]
pub enum SpellError {
    #[error("Spell {0} does not exist.")]
    DoesNotExist(String),
    #[error("Failed to contact API.")]
    ConnectionError,
    #[error("Failed to parse {0} from requested spell {1}")]
    ParseError(String, String),
}

pub async fn get_spell(name: &str) -> Result<Spell, SpellError> {
    let response_data = request_spell_page(name)
        .await
        .ok_or(SpellError::ConnectionError)?
        .spell
        .ok_or(SpellError::DoesNotExist(name.to_string()))?;

    let casting_time = response_data.casting_time.clone();
    let components = response_data
        .components
        .iter()
        .filter_map(|v| v.chars().next())
        .collect::<Vec<_>>();
    let concentration = response_data.concentration;
    let description = response_data.desc.clone();
    let duration = response_data.duration.clone();
    let level: usize = response_data
        .level
        .try_into()
        .map_err(|_| SpellError::ParseError("Spell Level".to_string(), name.to_string()))?;
    let material = response_data.material.clone();
    let name = response_data.name.clone();
    let ritual = response_data.ritual;
    let school_str = &response_data
        .school
        .as_ref()
        .ok_or(SpellError::ParseError(
            "Spell School".to_string(),
            name.to_string(),
        ))?
        .name;
    let school: School = School::from_str(school_str)
        .map_err(|_| SpellError::ParseError("Spell School Parsing".to_string(), name.clone()))?;
    let range = response_data.range.clone();
    let higher_level = response_data
        .higher_level
        .as_ref()
        .cloned()
        .unwrap_or(vec![]);

    let damage = process_damage(&response_data)?;
    let leveled_damage = process_leveled_damage(&response_data)?;

    Ok(Spell {
        casting_time,
        components,
        concentration,
        description,
        duration,
        level,
        material,
        name,
        ritual,
        school,
        range,
        higher_level,
        damage,
        leveled_damage,
    })
}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>>
where
    T: Clone,
{
    if v.is_empty() {
        return vec![];
    }
    (0..v[0].len())
        .map(|i| v.iter().map(|inner| inner[i].clone()).collect::<Vec<T>>())
        .collect()
}

fn process_damage(
    response_data: &spell_query::SpellQuerySpell,
) -> Result<Option<Vec<Vec<DamageRoll>>>, SpellError> {
    let name = &response_data.name;
    let damage = response_data
        .damage
        .as_ref()
        .map(|x| {
            x.iter()
                .map(|y| {
                    let damage_type_str = &y
                        .damage_type
                        .as_ref()
                        .ok_or(SpellError::ParseError(
                            "Damage Type".to_string(),
                            name.to_string(),
                        ))?
                        .name;
                    let damage_type = DamageType::from_str(damage_type_str).map_err(|_| {
                        SpellError::ParseError("Damage Type Parsing".to_string(), name.to_string())
                    })?;
                    let mut levels = y
                        .damage_at_slot_level
                        .iter()
                        .map(|z| (z.level, &z.value))
                        .map(|(level, dmg_string)| {
                            DamageRoll::from_str(dmg_string, damage_type).map(|v| (level, v))
                        })
                        .collect::<Option<Vec<_>>>()
                        .ok_or(SpellError::ParseError(
                            "Leveled Damage Roll Parsing".to_string(),
                            name.to_string(),
                        ))?;
                    levels.sort_by_key(|k| k.0);
                    Ok(levels
                        .into_iter()
                        .map(|(_, damage)| damage)
                        .collect::<Vec<_>>())
                })
                .collect::<Result<Vec<Vec<DamageRoll>>, SpellError>>()
        })
        .transpose()?
        .map(transpose);
    Ok(damage)
}

async fn request_spell_page(name: &str) -> Option<spell_query::ResponseData> {
    //get data
    let vars = spell_query::Variables {
        index: name.to_kebab_case(),
    };
    let request_body = SpellQuery::build_query(vars);

    let client = reqwest::Client::new();
    let res = client
        .post("https://www.dnd5eapi.co/graphql")
        .json(&request_body)
        .send()
        .await
        .ok()?;
    let response_body: Response<spell_query::ResponseData> = res.json().await.ok()?;
    response_body.data
}

// The api errors on leveled cantrips that can't be upcasted, so this is a hardcoded workaround
fn leveled_damage_edge_cases(
    response_data: &spell_query::SpellQuerySpell,
) -> Option<Vec<(usize, DamageRoll)>> {
    let name = &response_data.name;

    fn dr(dice: usize, damage_type: DamageType) -> Option<Vec<(usize, DamageRoll)>> {
        Some(vec![
            (
                1,
                DamageRoll {
                    number: 1,
                    dice,
                    bonus: 0,
                    damage_type,
                },
            ),
            (
                5,
                DamageRoll {
                    number: 2,
                    dice,
                    bonus: 0,
                    damage_type,
                },
            ),
            (
                11,
                DamageRoll {
                    number: 3,
                    dice,
                    bonus: 0,
                    damage_type,
                },
            ),
            (
                17,
                DamageRoll {
                    number: 4,
                    dice,
                    bonus: 0,
                    damage_type,
                },
            ),
        ])
    }
    match name.as_str() {
        "Acid Splash" => dr(6, DamageType::Acid),
        "Chill Touch" => dr(8, DamageType::Necrotic),
        "Fire Bolt" => dr(10, DamageType::Fire),
        "Poison Spray" => dr(12, DamageType::Poison),
        "Produce Flame" => dr(8, DamageType::Fire),
        "Ray of Frost" => dr(8, DamageType::Cold),
        "Sacred Flame" => dr(8, DamageType::Radiant),
        "Shocking Grasp" => dr(8, DamageType::Lightning),
        "Vicious Mockery" => dr(4, DamageType::Psychic),
        _ => None,
    }
}

fn process_leveled_damage(
    response_data: &spell_query::SpellQuerySpell,
) -> Result<Option<Vec<(usize, DamageRoll)>>, SpellError> {
    let name = &response_data.name;
    if let Some(s) = leveled_damage_edge_cases(response_data) {
        return Ok(Some(s));
    }
    let damage = match response_data.damage.as_ref().map(|v| v.first()) {
        Some(Some(v)) => v,
        _ => return Ok(None),
    };

    let damage_string = &damage
        .damage_type
        .as_ref()
        .ok_or(SpellError::ParseError(
            "Leveled Damage Type".to_string(),
            name.clone(),
        ))?
        .name;
    let damage_type = DamageType::from_str(damage_string).map_err(|_| {
        SpellError::ParseError("Leveled Damage Type Parsing".to_string(), name.clone())
    })?;

    let damage_at_character_level = match damage.damage_at_character_level.as_ref() {
        Some(v) => v,
        _ => return Ok(None),
    };

    Ok(Some(
        damage_at_character_level
            .iter()
            .map(|v| {
                let level: usize = v.level.try_into().map_err(|_| {
                    SpellError::ParseError("Damage Character Level".to_string(), name.clone())
                })?;
                let damage_roll = DamageRoll::from_str(&v.value, damage_type).ok_or(
                    SpellError::ParseError("Damage Character Roll".to_string(), name.clone()),
                )?;
                Ok((level, damage_roll))
            })
            .collect::<Result<Vec<(usize, DamageRoll)>, SpellError>>()?,
    ))
}
