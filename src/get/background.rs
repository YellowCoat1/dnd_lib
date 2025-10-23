use serde_json::Value;
use super::get_page::get_raw_json;
use super::json_tools::{ValueError, ValueExt};
use super::item::get_item;
use crate::character::features::Feature;
use crate::character::{
    background::Background,
    stats::SkillType,
    features::PresentedOption,
};
use crate::get::json_tools::parse_string;

pub async fn get_background(name: &str) -> Result<Background, ValueError> {
    let index = parse_string(name);
    let json = get_raw_json(format!("backgrounds/{index}")).await?;

    let proficiencies = json.get_array("starting_proficiencies")?
        .iter().map(|v| {
            SkillType::from_name(&v.get_str("name")?[7..])
                .map(|u|  PresentedOption::Base(u))
                .map_err(|_| ValueError::ValueMismatch(String::from("starting proficiency")))
        }).collect::<Result<Vec<PresentedOption<SkillType>>, ValueError>>()?;

    let equipment_array = json.get_array("starting_equipment")?;
    let mut equipment = Vec::with_capacity(equipment_array.len());

    for equipment_val in equipment_array {
        let equipment_index = equipment_val
            .get_map("equipment")?
            .get_str("index")?;
        let item_val = get_item(&equipment_index).await
            .map_err(|_| ValueError::ValueMismatch(String::from("item")))?;
        let equipment_num = equipment_val.get_usize("quantity")?;
        equipment.push((item_val, equipment_num));
    }

    let feature_map = json.get_map("feature")?;
    let feature_desc = feature_map.get_array("desc")?
        .iter()
        .map(|v| v.as_string("feature description"))
        .collect::<Result<Vec<String>, ValueError>>()?;
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
            Ok(PresentedOption::Base(v.get_str("desc")?))
        }).collect::<Result<Vec<PresentedOption<String>>, ValueError>>()?;

    let ideals = PresentedOption::Choice(ideals_vec);
    
    Ok(Background {
        proficiencies,
        equipment,
        features: vec![feature],
        personality_traits,
        ideals,
        bonds,
        flaws,
    })
}

fn process_personality(json: &Value) -> Result<PresentedOption<String>, ValueError> {
    let array = json.get_map("from")?.get_array("options")?
        .iter()
        .map(|v| {
            Ok(PresentedOption::Base(v.get_str("string")?))
        }).collect::<Result<Vec<PresentedOption<String>>, ValueError>>()?;

    Ok(PresentedOption::Choice(array))
}


#[cfg(test)]
mod test {
    use super::*;

    // literally the only background in the api, but whatever
    #[tokio::test]
    async fn get_acolyte() {
        let acolyte = get_background("acolyte").await.expect("failed to get acolyte!");
        let insight = acolyte.proficiencies.get(0).expect("acolyte should have proficiencies!");
        assert_eq!(*insight, PresentedOption::Base(SkillType::Insight));
        let hero = acolyte.personality_traits.choices().unwrap().get(0).expect("acolyte should have personality traits!");
        assert_eq!(*hero, PresentedOption::Base(String::from("I idolize a particular hero of my faith, and constantly refer to that person's deeds and example.")));
        let tradition = acolyte.ideals.choices().unwrap().get(0).expect("acolyte should have ideals!");
        assert_eq!(*tradition, PresentedOption::Base(String::from("Tradition. The ancient traditions of worship and sacrifice must be preserved and upheld.")));
    }
}
