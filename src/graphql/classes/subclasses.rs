use crate::{
    graphql::classes::{class_query::ClassQueryClass, ClassError},
    rules2014::{
        class::Subclass,
        features::{Feature, FeatureEffect, PresentedOption},
    },
};

pub fn process_subclasses(response_data: &ClassQueryClass) -> Result<Vec<Subclass>, ClassError> {
    let name = &response_data.name;
    response_data
        .subclasses
        .as_ref()
        .ok_or(ClassError::parse_error("", name))?
        .iter()
        .map(|subclass| {
            Ok(Subclass {
                name: subclass.name.clone(),
                description: subclass.desc.clone(),
                features: subclass_features(
                    name,
                    subclass
                        .subclass_levels
                        .as_ref()
                        .ok_or(ClassError::parse_error("subclass levels", name))?,
                )?,
            })
        })
        .collect::<Result<Vec<Subclass>, ClassError>>()
}

type Levels = Vec<super::class_query::ClassQueryClassSubclassesSubclassLevels>;
fn subclass_features(
    name: &str,
    levels: &Levels,
) -> Result<[Vec<PresentedOption<Feature>>; 20], ClassError> {
    let mut base: [Vec<PresentedOption<Feature>>; 20] = Default::default();
    let levels_iter = levels
        .iter()
        .map(|v| {
            v.features
                .as_ref()
                .map(|z| z.iter().map(|v2| (v.level, v2)))
                .ok_or(ClassError::parse_error("subclass features", name))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten();

    for (level, subclass_feature) in levels_iter {
        let feature = Feature {
            name: subclass_feature.name.clone(),
            description: subclass_feature.desc.clone(),
            effects: subclass_feature_effects(&subclass_feature.name),
        };
        let level_usize: usize = level
            .try_into()
            .map_err(|_| ClassError::parse_error("subclass feature level", name))?;
        base.get_mut(level_usize - 1)
            .ok_or(ClassError::parse_error(
                format!("invalid feature level {level_usize}").as_str(),
                name,
            ))?
            .push(PresentedOption::Base(feature));
    }
    Ok(base)
}

fn subclass_feature_effects(name: &str) -> Vec<FeatureEffect> {
    match name {
        "Bonus Proficiency" => vec![FeatureEffect::ArmorProficiency(
            crate::rules2014::items::ArmorCategory::Heavy,
        )],
        _ => vec![],
    }
}
