use super::{
    get_page::get_raw_json,
    json_tools::{parse_string, string_array, ValueExt},
};
use crate::get::json_tools::value_name;
use crate::getter::CharacterDataError;
use crate::rules2014::features::Feature;
use crate::rules2014::{class::Subclass, features::PresentedOption};

pub async fn get_subclass(name: &str) -> Result<Subclass, CharacterDataError> {
    let index = parse_string(name);
    let json = get_raw_json(format!("subclasses/{index}")).await?;
    let levels = get_raw_json(format!("subclasses/{index}/levels")).await?;

    let api_getter = super::Dnd5eapigetter::new();

    let name = json.get_str("name")?;
    let description = string_array(json.get_array("desc")?)?;

    let levels_arr = levels
        .as_array()
        .ok_or_else(|| CharacterDataError::mismatch("levels json", "array", value_name(&levels)))?;

    let mut features: [Vec<PresentedOption<Feature>>; 20] = Default::default();

    for level_object in levels_arr.iter() {
        let level_number = level_object.get_usize("level")?;
        let features_arr = level_object.get_array("features")?;

        let mut features_vec = Vec::with_capacity(features_arr.len());
        for feature_obj in features_arr {
            let index = feature_obj.get_str("index")?;
            let feature = api_getter.get_feature(&index).await?;
            features_vec.push(PresentedOption::Base(feature));
        }

        features[level_number - 1] = features_vec;
    }

    Ok(Subclass {
        name,
        description,
        features,
    })
}

#[cfg(test)]
mod tests {
    use crate::rules2014::features::PresentedOption;

    use super::get_subclass;

    #[tokio::test]
    async fn retrieve_subclass() {
        let champion = get_subclass("champion").await.unwrap();
        assert_eq!(champion.name, "Champion");
        let improved_critical = match champion.features[2]
            .first()
            .expect("champion should have a third level feature")
        {
            PresentedOption::Base(b) => b,
            _ => panic!("feature was an unexpected type!"),
        };
        assert_eq!(improved_critical.name, "Improved Critical");
    }
}
