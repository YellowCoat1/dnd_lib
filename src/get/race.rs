use serde_json::Value;
use crate::character::stats::Size;
use crate::character::{Race, Subrace};
use crate::character::features::PresentedOption;
use super::get_page::get_raw_json;
use super::json_tools::{parse_string, ValueExt};
use crate::getter::CharacterDataError;
use super::feature::get_feature_from_trait;
use super::subrace::get_subrace;

// the func to run through ability bonuses is in subrace, since that module isn't publicly exported
use super::subrace::process_ability_bonuses;

pub async fn get_race(name: &str) -> Result<Race, CharacterDataError> {
    let index = parse_string(name);
    get_race_raw(index).await
}

async fn get_race_raw(index_name: String) -> Result<Race, CharacterDataError> {

    let race_json = get_raw_json(format!("races/{index_name}")).await?;

    let name = race_json.get_str("name")?;
    let speed: usize = race_json.get_usize("speed")?;
    let size_string = race_json.get_str("size")?;
    let size = process_size(&size_string)
        .ok_or_else(||CharacterDataError::mismatch("size", "Valid size string", "Invalid size string"))?;

    let ability_bonuses_array= race_json.get_array("ability_bonuses")?;
    let ability_bonuses = process_ability_bonuses(ability_bonuses_array)?;

    let languages_array = race_json.get_array("languages")?;
    let languages = process_languages(languages_array)?;

    let traits_arr = race_json.get_array("traits")?;
    let mut traits = Vec::with_capacity(traits_arr.len());

    for traits_val in traits_arr.iter() {
        let index = traits_val.get_str("index")?;
        let feature = get_feature_from_trait(&index).await?;
        traits.push(feature);
    }


    let subrace_array = race_json.get_array("subraces")?;
    let subraces_raw = process_subraces(subrace_array).await?;
    let subraces = PresentedOption::Choice(subraces_raw.into_iter().collect());
    Ok(Race {
        name,
        size,
        speed,
        ability_bonuses,
        traits,
        languages,
        subraces,
    })
}

fn process_languages(arr: &[Value]) -> Result<Vec<String>, CharacterDataError> {
    let mut languages = vec![];

    for language  in arr.iter() {
         let language_name = language.get_str("name")?;        
        languages.push(language_name);
    }

    Ok(languages)
}

async fn process_subraces(arr: &[Value]) -> Result<Vec<Subrace>, CharacterDataError> {
    let mut subraces = Vec::with_capacity(arr.len());
    for val in arr {
        let name = val.get_str("index")?;
        let subrace = get_subrace(&name).await?;
        subraces.push(subrace);
    }
    Ok(subraces)
}

fn process_size(s: &str) -> Option<Size> {
    match s {
        "Tiny" => Some(Size::Tiny),
        "Small" => Some(Size::Small),
        "Medium" => Some(Size::Medium),
        "Large" => Some(Size::Large),
        "Huge" => Some(Size::Huge),
        "Gargantuan" => Some(Size::Gargantuan),
        _ => None
    }
}
