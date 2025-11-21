use super::get_page::get_raw_json;
use super::json_tools::ValueExt;
use crate::character::background::LanguageOption;
use crate::character::features::Feature;
use crate::character::{background::Background, features::PresentedOption, stats::SkillType};
use crate::get::json_tools::{parse_skilltype, parse_string};
use crate::getter::CharacterDataError;
use crate::getter::DataProvider;
use serde_json::Value;

pub const BACKGROUND_NAMES: [&str; 1] = ["acolyte"];

pub async fn get_background(
    getter: &impl DataProvider,
    name: &str,
) -> Result<Background, CharacterDataError> {
    let index = parse_string(name);
    let json = get_raw_json(format!("backgrounds/{index}")).await?;

    let name = json.get_str("index")?;

    let proficiencies = json
        .get_array("starting_proficiencies")?
        .iter()
        .map(|v| {
            let name = &v.get_str("name")?[7..];
            parse_skilltype("Background proficiencies", name).map(PresentedOption::Base)
        })
        .collect::<Result<Vec<PresentedOption<SkillType>>, CharacterDataError>>()?;

    let equipment_array = json.get_array("starting_equipment")?;
    let mut equipment = Vec::with_capacity(equipment_array.len());

    for equipment_val in equipment_array {
        let equipment_index = equipment_val.get_map("equipment")?.get_str("index")?;
        let item_val = getter.get_item(&equipment_index).await?;
        let equipment_num = equipment_val.get_usize("quantity")?;
        equipment.push((item_val, equipment_num).into());
    }

    let feature_map = json.get_map("feature")?;
    let feature_desc = feature_map
        .get_array("desc")?
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

    let ideals = json
        .get_map("ideals")?
        .get_map("from")?
        .get_array("options")?
        .iter()
        .map(|v| v.get_str("desc"))
        .collect::<Result<Vec<String>, CharacterDataError>>()?;

    // hardcoding languages. Acolyte background gives two languages of choice.
    let language_options: Vec<LanguageOption> =
        vec![LanguageOption::UnnamedChoice, LanguageOption::UnnamedChoice];

    Ok(Background {
        name,
        proficiencies,
        equipment,
        features: vec![feature],
        personality_traits,
        language_options,
        ideals,
        bonds,
        flaws,
    })
}

fn process_personality(json: &Value) -> Result<Vec<String>, CharacterDataError> {
    json.get_map("from")?
        .get_array("options")?
        .iter()
        .map(|v| v.get_str("string"))
        .collect::<Result<Vec<String>, CharacterDataError>>()
}
