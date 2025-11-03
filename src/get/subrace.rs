use super::json_tools::ValueExt;
use crate::getter::CharacterDataError;
use super::get_page::get_raw_json;
use serde_json::Value;
use crate::character::Subrace;
use crate::character::stats::StatType;
use crate::get::json_tools::parse_string;
use super::feature::get_feature_from_trait;

pub async fn get_subrace(name: &str) -> Result<Subrace, CharacterDataError> {
    let index = parse_string(name);

    let json = get_raw_json(format!("subraces/{index}")).await?;

    let name = json.get_str("name")?;
    let description = json.get_str("desc")?;

    let ability_bonus_array = json.get_array("ability_bonuses")?;
    let ability_bonuses = process_ability_bonuses(ability_bonus_array)?;

    
    let traits_arr = json.get_array("racial_traits")?;
    let mut traits = Vec::with_capacity(traits_arr.len());
    for traits_val in traits_arr.iter() {
        let trait_index = traits_val.get_str("index")?;
        let feature = get_feature_from_trait(&trait_index).await?;
        traits.push(feature);
    }

    Ok(Subrace {
        name,
        description,
        ability_bonuses,
        traits,
    })
}

pub fn process_ability_bonuses(arr: &[Value]) -> Result<Vec<(StatType, isize)>, CharacterDataError> {
    let mut ability_bonuses: Vec<(StatType, isize)> = vec![];

    for ability_bonus in arr.iter() {

        let ability_score_map = ability_bonus.get_map("ability_score")?;
        let ability_score_name = ability_score_map.get_str("name")?;
        let ability_score_bonus: isize = ability_bonus.get_usize("bonus")?.try_into().unwrap();

        let stat_type = StatType::from_shorthand(ability_score_name.as_str())
            .ok_or_else(|| CharacterDataError::mismatch("ability score name", "StatType string", "improper StatType string"))?;

        ability_bonuses.push((stat_type, ability_score_bonus));
    }

    Ok(ability_bonuses)
}


