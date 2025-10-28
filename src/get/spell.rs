use serde_json::Value;
use crate::character::spells::Spell;
use crate::character::items::DamageRoll;
use crate::get::json_tools::parse_string;
use super::get_page::get_raw_json;
use super::json_tools::{ValueExt, ValueError, string_array};

pub async fn get_spell(name: &str) -> Result<Spell, ValueError> {
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
    let school = json.get_map("school")?.get_str("name")?.as_str().parse()
        .map_err(|_| ValueError::ValueMismatch("spell school".to_string()))?;
    let components: Vec<char> = string_array(json.get_array("components")?)?
        .iter()
        .map(|v| v.chars().next().unwrap())
        .collect();
    let material = json.get_str("material").ok();
    let damage = spell_damage(json.get_map("damage").ok())?;
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
    })
}


fn spell_damage(v: Option<&Value>) -> Result<Option<Vec<Vec<DamageRoll>>>, ValueError> {
    v.map(|v| {
        let damage_type_name =  v.get_map("damage_type")?.get_str("name")?;
        let damage_type = damage_type_name.parse()
            .map_err(|_| ValueError::ValueMismatch("damage type".to_string()))?;

        let damage_map = v
            .get_map("damage_at_slot_level")?
            .as_object()
            .ok_or(ValueError::ValueMismatch("damage value".to_string()))?;

        let damage_strings = damage_map
            .values()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| ValueError::ValueMismatch("damage value".to_string()))
            })
            .collect::<Result<Vec<_>,_>>()?;

        let damage_vec = damage_strings
            .iter()
            .map(|v| {
                DamageRoll::from_str(v, damage_type)
                    .ok_or_else(|| ValueError::ValueMismatch("damage value".to_string()))
                    .map(|v| vec![v])
            })
            .collect::<Result<Vec<_>,_>>()?;
        Ok(damage_vec)
    }).transpose()
}

#[cfg(test)]
mod tests {
    use crate::character::items::{DamageRoll, DamageType};

    use super::get_spell;

    #[tokio::test]
    pub async fn spell_retrieval() {
        let acid_arrow = get_spell("acid-arrow").await.expect("failed to get spell");
        assert_eq!(acid_arrow.name, "Acid Arrow");
        assert_eq!(acid_arrow.range, "90 feet");
        let damage = acid_arrow.damage.expect("acid arrow should have damage!");
        let acid = DamageType::Acid;
        let second_level_damage = damage.first().expect("acid arrow should have 2nd level damage!");
        assert_eq!(second_level_damage[0], DamageRoll::new(4, 4, acid));
        let ninth_level_damage = damage.get(7).expect("acid arrow should have 9th level damage!");
        assert_eq!(ninth_level_damage[0], DamageRoll::new(11, 4, acid));
    }
}
