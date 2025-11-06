use serde_json::Value;
use super::get_page::get_raw_json;
use crate::getter::CharacterDataError;
use crate::getter::DataProvider;
use super::json_tools::ValueExt;
use crate::character::features::Feature;
use crate::character::{
    Background,
    stats::SkillType,
    features::PresentedOption,
};
use crate::get::json_tools::parse_string;

/// Gets a background from the api.
///
/// There's only one background available through dnd5eapi: acolyte.
pub async fn get_background(getter: &impl DataProvider, name: &str) -> Result<Background, CharacterDataError> {
    let index = parse_string(name);
    let json = get_raw_json(format!("backgrounds/{index}")).await?;

    let name = json.get_str("index")?;

    let proficiencies = json.get_array("starting_proficiencies")?
        .iter().map(|v| {
            SkillType::from_name(&v.get_str("name")?[7..])
                .ok_or_else(|| CharacterDataError::mismatch("starting proficiencies", "Valid SkillType string", "Invalid SkillType string"))
                .map(PresentedOption::Base)
        }).collect::<Result<Vec<PresentedOption<SkillType>>, CharacterDataError>>()?;

    let equipment_array = json.get_array("starting_equipment")?;
    let mut equipment = Vec::with_capacity(equipment_array.len());

    for equipment_val in equipment_array {
        let equipment_index = equipment_val
            .get_map("equipment")?
            .get_str("index")?;
        let item_val = getter.get_item(&equipment_index).await?;
        let equipment_num = equipment_val.get_usize("quantity")?;
        equipment.push((item_val, equipment_num));
    }

    let feature_map = json.get_map("feature")?;
    let feature_desc = feature_map.get_array("desc")?
        .iter()
        .map(|v| v.as_string("feature description"))
        .collect::<Result<Vec<String>, CharacterDataError>>()?;
    let feature = Feature {
        name: feature_map.get_str("name")?,
        description: feature_desc,
        effects: vec![],
    };

    let personality_traits = process_personality(json.get_map("personality_traits")?)?;
    let bonds = process_personality(json.get_map("bonds")?)?;
    let flaws = process_personality(json.get_map("flaws")?)?;

    let ideals_vec = json.get_map("ideals")?
        .get_map("from")?.get_array("options")?
        .iter()
        .map(|v| {
            v.get_str("desc")
        }).collect::<Result<Vec<String>, CharacterDataError>>()?;

    let ideals = PresentedOption::Choice(ideals_vec);
    
    Ok(Background {
        name,
        proficiencies,
        equipment,
        features: vec![feature],
        personality_traits,
        ideals,
        bonds,
        flaws,
    })
}

fn process_personality(json: &Value) -> Result<PresentedOption<String>, CharacterDataError> {
    let array = json.get_map("from")?.get_array("options")?
        .iter()
        .map(|v| {
            v.get_str("string")
        }).collect::<Result<Vec<String>, CharacterDataError>>()?;

    Ok(PresentedOption::Choice(array))
}



