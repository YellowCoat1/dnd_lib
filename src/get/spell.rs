use super::get_page::get_raw_json;
use super::json_tools::{string_array, value_name, ValueExt};
use crate::rules2014::items::{DamageRoll, DamageType};
use crate::rules2014::spells::Spell;
use crate::get::json_tools::parse_string;
use crate::getter::CharacterDataError;
use serde_json::Value;

type StandardDamage = Vec<Vec<DamageRoll>>;
type LeveledDamage = Vec<(usize, DamageRoll)>;

pub async fn get_spell(name: &str) -> Result<Spell, CharacterDataError> {
    let index = parse_string(name);

    let json = get_raw_json(format!("spells/{index}")).await?;
    let name = json.get_str("name")?;
    let description = string_array(json.get_array("desc")?)?;
    let higher_level = string_array(json.get_array("higher_level")?)?;
    let ritual = json.get_bool("ritual")?;
    let concentration = json.get_bool("concentration")?;
    let casting_time = json.get_str("casting_time")?;
    let level = json.get_usize("level")?;
    let range = json.get_str("range")?;
    let school = json
        .get_map("school")?
        .get_str("name")?
        .as_str()
        .parse()
        .map_err(|_| {
            CharacterDataError::mismatch(
                "spell school",
                "Valid spell school name",
                "Invalid spell school name",
            )
        })?;
    let components: Vec<char> = string_array(json.get_array("components")?)?
        .iter()
        .map(|v| v.chars().next().unwrap())
        .collect();
    let material = json.get_str("material").ok();
    let (damage, leveled_damage) = spell_damage(json.get_map("damage").ok())?;
    let duration = json.get_str("duration")?;

    Ok(Spell {
        name,
        description,
        higher_level,
        ritual,
        concentration,
        casting_time,
        duration,
        level,
        range,
        school,
        components,
        material,
        damage,
        leveled_damage,
    })
}

fn spell_damage(
    v: Option<&Value>,
) -> Result<(Option<StandardDamage>, Option<LeveledDamage>), CharacterDataError> {
    v.map(|v| {
        let damage_type_name = v.get_map("damage_type")?.get_str("name")?;
        let damage_type: DamageType = damage_type_name.parse().map_err(|_| {
            CharacterDataError::mismatch(
                "spell damage type",
                "Valid DamageType string",
                "Invalid DamageType string",
            )
        })?;

        let damage_vec: Option<StandardDamage> = v
            .get_map("damage_at_slot_level")
            .ok()
            .map(|v| standard_spell_damage(damage_type, v))
            .transpose()?;
        let leveled_damage_map: Option<LeveledDamage> = v
            .get_map("damage_at_character_level")
            .ok()
            .map(|v| leveled_spell_damage(damage_type, v))
            .transpose()?
            // sorts the damage by level
            .map(|mut v| {
                v.sort_by(|a, b| a.0.cmp(&b.0));
                v
            });

        Ok((damage_vec, leveled_damage_map))
    })
    .transpose()
    .map(|v| v.unwrap_or((None, None)))
}

fn standard_spell_damage(
    damage_type: DamageType,
    json: &Value,
) -> Result<StandardDamage, CharacterDataError> {
    json.as_object()
        .ok_or_else(|| CharacterDataError::mismatch("spell damage", "Object", value_name(json)))?
        .values()
        .map(|v| {
            v.as_str().ok_or_else(|| {
                CharacterDataError::mismatch("spell damage", "string", value_name(v))
            })
        })
        .collect::<Result<Vec<_>, CharacterDataError>>()?
        .into_iter()
        .map(|v| {
            DamageRoll::from_str(v, damage_type)
                .ok_or_else(|| {
                    CharacterDataError::mismatch(
                        "spell damage",
                        "valid DamageRoll string",
                        "invalid DamageRoll string",
                    )
                })
                .map(|v| vec![v])
        })
        .collect::<Result<Vec<_>, _>>()
}

fn leveled_spell_damage(
    damage_type: DamageType,
    json: &Value,
) -> Result<LeveledDamage, CharacterDataError> {
    json.as_object()
        .ok_or_else(|| CharacterDataError::mismatch("spell damage", "Object", value_name(json)))?
        .iter()
        .map(|(level, damage)| {
            let level_num: usize = level.parse().map_err(|_| {
                CharacterDataError::mismatch(
                    "Cantrip damage level",
                    "number",
                    "invalid string to parse",
                )
            })?;
            let damage_string = damage.as_str().ok_or_else(|| {
                CharacterDataError::mismatch("Cantrip damage", "String", value_name(damage))
            })?;
            let damage_roll =
                DamageRoll::from_str(damage_string, damage_type).ok_or_else(|| {
                    CharacterDataError::mismatch(
                        "Cantrip damage roll",
                        "DamageRoll valid string",
                        "DamageRoll invalid string",
                    )
                })?;
            Ok((level_num, damage_roll))
        })
        .collect::<Result<Vec<_>, _>>()
}

#[cfg(test)]
mod tests {
    use crate::rules2014::items::{DamageRoll, DamageType};

    use super::get_spell;

    #[tokio::test]
    async fn spell_retrieval() {
        let acid_arrow = get_spell("acid-arrow").await.expect("failed to get spell");
        assert_eq!(acid_arrow.name, "Acid Arrow");
        assert_eq!(acid_arrow.range, "90 feet");
        let damage = acid_arrow.damage.expect("acid arrow should have damage!");
        let acid = DamageType::Acid;
        let second_level_damage = damage
            .first()
            .expect("acid arrow should have 2nd level damage!");
        assert_eq!(second_level_damage[0], DamageRoll::new(4, 4, acid));
        let ninth_level_damage = damage
            .get(7)
            .expect("acid arrow should have 9th level damage!");
        assert_eq!(ninth_level_damage[0], DamageRoll::new(11, 4, acid));
    }

    #[tokio::test]
    async fn poison_spray() {
        use DamageType::Poison;
        let poison_spray = get_spell("poison spray")
            .await
            .expect("failed to get poison spray");
        let poison_spray_damage = match poison_spray.leveled_damage {
            Some(s) => s,
            _ => panic!("Poison spray should have leveled damage"),
        };

        assert_eq!(
            poison_spray_damage,
            vec![
                (1, DamageRoll::new(1, 12, Poison)),
                (5, DamageRoll::new(2, 12, Poison)),
                (11, DamageRoll::new(3, 12, Poison)),
                (17, DamageRoll::new(4, 12, Poison))
            ]
        );
    }
}
