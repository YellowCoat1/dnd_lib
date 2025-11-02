use regex::Regex;
use serde_json::Value;
use crate::character::features::{PresentedOption, Feature, FeatureEffect, AbilityScoreIncrease};
use crate::character::stats::StatType;
use super::get_page::get_raw_json;
use super::json_tools::{choice, parse_string, ValueExt};
use crate::getter::CharacterDataError;


pub async fn get_feature(name: &str) -> Result<Feature, CharacterDataError> {
    let index = parse_string(name);
    get_feature_raw(index).await
}

pub async fn get_feature_raw(index_name: String) -> Result<Feature, CharacterDataError> {
    let item_json = get_raw_json(format!("features/{index_name}")).await?;

    let name = item_json.get_str("name")?;

    let description_arr = item_json.get_array("desc")?;

    let description: Vec<String> = description_arr.iter().map(|v| match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(CharacterDataError::ValueMismatch(String::from("description line"))),
    }).collect::<Result<Vec<String>, CharacterDataError>>()?;

    let effects = feature_effects(&index_name);


    let feature = Feature {
        name,
        description,
        effects,
    };


    Ok(feature)
}

pub async fn get_feature_from_trait(index_name: &str) -> Result<PresentedOption<Feature>, ()> {
    get_feature_from_trait_other(index_name).await.map_err(|_| ())
}

async fn get_feature_from_trait_other(index_name: &str) -> Result<PresentedOption<Feature>, CharacterDataError> {

    let trait_json = get_raw_json(format!("traits/{index_name}")).await?;
    
    // draconic ancestry is another beast, and it deserves it's own function.
    if index_name.to_lowercase() == "draconic-ancestry" {
        return get_draconic_ancestry(trait_json).await
    }

    let name = trait_json.get_str("name")?;
    let description_arr = trait_json.get_array("desc")?;

    let description: Vec<String> = description_arr
        .iter()
        .map(|v| v.as_string("description"))
        .collect::<Result<Vec<String>, CharacterDataError>>()?;
 
    let feature = Feature {
        name,
        description,
        effects: feature_effects(index_name),
    };

    Ok(PresentedOption::Base(feature))
}


async fn get_draconic_ancestry(json: Value) -> Result<PresentedOption<Feature>, CharacterDataError> {
    let trait_specific = json.get_map("trait_specific")?;

    let subtrait_options = match trait_specific.get("subtrait_options") {
        Some(v) => v,
        _ => return Err(CharacterDataError::ValueMismatch(String::from("subtrait_options"))),
    };

    let (_, _, trait_option) = choice(subtrait_options)
        .map_err(|_| CharacterDataError::ValueMismatch(String::from("draconic choice")))?;

    let v = trait_option.map_async(|m| async {
        let item_map = match m.get("item") {
            Some(v) => v,
            _ => return Err(CharacterDataError::ValueMismatch(String::from("item"))),
        };

        let index = item_map.get_str("index")?;

        let json = get_raw_json(format!("traits/{index}")).await?;

        let name = json.get_str("name")?;

        let mut desc: Vec<String> = json.get_array("desc")?
            .iter()
            .map(|v| v.as_string("description"))
            .collect::<Result<Vec<String>, CharacterDataError>>()?;
        
        let trait_specific_map = json.get_map("trait_specific")?;

        let breath_weapon_map = trait_specific_map.get_map("breath_weapon")?;
        let breath_weapon_desc = breath_weapon_map.get_str("desc")?;

        let breath_weapon_aoe_map = breath_weapon_map.get_map("area_of_effect")?;
        let breath_weapon_size = breath_weapon_aoe_map.get_usize("size")?;
        let breath_weapon_type = breath_weapon_aoe_map.get_str("type")?;

        desc.push(breath_weapon_desc);
        desc.push(format!("type {breath_weapon_type} of size {breath_weapon_size}"));

        Ok(Feature {
            name,
            description: desc,
            effects: vec![],
        })
    }).await.map(|v| v.unwrap());
    Ok(v)
}

fn feature_effects(index_name: &str) -> Vec<FeatureEffect> {

    if matches_ability_score_increase(index_name) {
        return vec![FeatureEffect::AbilityScoreIncrease(AbilityScoreIncrease::Unchosen)]
    } else if matches_expertise(index_name) {
        return vec![FeatureEffect::Expertise([None, None])];
    } 
    
    match index_name {
        "barbarian-unarmored-defense" => vec![FeatureEffect::UnarmoredDefense(10, StatType::Dexterity, Some(StatType::Constitution))],
        "monk-unarmored-defense" => vec![FeatureEffect::UnarmoredDefense(10, StatType::Dexterity, Some(StatType::Wisdom))],
        "draconic-resilience" => vec![FeatureEffect::LeveledHpIncrease, FeatureEffect::UnarmoredDefense(13, StatType::Dexterity, None)],
        "dwarven-toughness" => vec![FeatureEffect::LeveledHpIncrease],
        _ => vec![]
    }

}

fn matches_ability_score_increase(string: &str) -> bool {
    Regex::new(r"^(.*)-ability-score-improvement-(\d+)$").unwrap().is_match(string)
}

fn matches_expertise(string: &str) -> bool {
    Regex::new(r"^(.*)-expertise-(\d+)$").unwrap().is_match(string)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_trait() {
        let feature_option = get_feature_from_trait("darkvision").await.unwrap();
        let feature = match feature_option {
            PresentedOption::Base(b) => b,
            PresentedOption::Choice(_) => panic!("Should just be one feature"),
        };
        assert_eq!(feature.name, "Darkvision");
        assert_eq!(feature.description[0], "You have superior vision in dark and dim conditions. You can see in dim light within 60 feet of you as if it were bright light, and in darkness as if it were dim light. You cannot discern color in darkness, only shades of gray.");
    }

    #[tokio::test]
    async fn test_draconic() {
        let draconic_ancestry = get_feature_from_trait("draconic-ancestry").await.unwrap();
        let first = match &draconic_ancestry.choices().unwrap()[0] {
            PresentedOption::Base(b) => b,
            _ => panic!("invalid draconic formatting"),
        };

        let tenth = match &draconic_ancestry.choices().unwrap()[9] {
            PresentedOption::Base(b) => b,
            _ => panic!("invalid draconic formatting"),
        };

        assert_eq!(first.name, "Draconic Ancestry (Black)");
        assert_eq!(tenth.name, "Draconic Ancestry (White)");
        assert_eq!(tenth.description[2], "type cone of size 15");
    }
}
