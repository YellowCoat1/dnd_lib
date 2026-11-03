use crate::{
    graphql::classes::ClassError, rules2014::{features::{AbilityScoreIncrease, Feature, FeatureEffect, PresentedOption}, stats::StatType},
};

use super::class_query::ClassQueryClass;
pub fn features(
    response_data: &ClassQueryClass,
) -> Result<[Vec<PresentedOption<Feature>>; 20], ClassError> {
    let name = &response_data.name;
    response_data
        .class_levels
        .iter()
        .map(|level| {
            level
                .features
                .as_ref()
                .ok_or(ClassError::parse_error("level features", name))
        })
        .map(|v| {
            v.map(|level_features| {
                // process all the features for 1 level
                level_features
                    .iter()
                    .map(|v| construct_feature(name, v))
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Result<Vec<_>, ClassError>>()
        .and_then(|z| {
            // convert from a Vec<Vec<T>> to a [Vec<T>; 20]
            z.try_into()
                .map_err(|_| ClassError::parse_error("feature levels len", name))
        })
}

fn construct_feature(
    class_name: &str,
    feature: &super::class_query::ClassQueryClassClassLevelsFeatures,
) -> PresentedOption<Feature> {
    PresentedOption::Base(Feature {
        name: feature.name.clone(),
        description: feature.desc.clone(),
        effects: effects_from_name(class_name, &feature.name),
    })
}

fn effects_from_name(class_name: &str, feature_name: &str) -> Vec<FeatureEffect> {
    match (class_name, feature_name) {
        (_, "Ability Score Improvement") => vec![FeatureEffect::AbilityScoreIncrease(
            AbilityScoreIncrease::Unchosen,
        )],
        (_, "Unarmored Movement") => vec![FeatureEffect::UnarmoredMovement],
        (_, "Expertise") => vec![FeatureEffect::Expertise([None, None])],
        ("Barbarian", "Unarmored Defense") =>vec![FeatureEffect::UnarmoredDefense(10, StatType::Dexterity, Some(StatType::Constitution))],
        ("Monk", "Unarmored Defense") =>vec![FeatureEffect::UnarmoredDefense(10, StatType::Dexterity, Some(StatType::Wisdom))],
        _ => vec![],
    }
}
