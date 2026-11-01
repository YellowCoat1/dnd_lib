use std::collections::HashMap;

use crate::graphql::classes::{class_query::ClassQueryClass, ClassError};

pub fn process_class_specific(
    response_data: &ClassQueryClass,
) -> Result<Vec<(String, [String; 20])>, ClassError> {
    let name = &response_data.name;
    let class_specific_strings: [Vec<(String, String)>; 20] = response_data
        .class_levels
        .iter()
        .map(|level| {
            level
                .class_specific
                .as_ref()
                .ok_or(ClassError::parse_error("class specific", name))
        })
        .map(|v| v.map(class_specific_from_data))
        .collect::<Result<Vec<_>, ClassError>>()?
        .try_into()
        .map_err(|_| ClassError::parse_error("class specific levels != 20 levels", name))?;

    transpose(name, class_specific_strings)
}

type ClassSpecificList = super::class_query::ClassQueryClassClassLevelsClassSpecific;
fn class_specific_from_data(class_specific_fields: &ClassSpecificList) -> Vec<(String, String)> {
    let mut vec = vec![];

    // barbarian
    if let Some(rages) = &class_specific_fields.rage_count {
        vec.push(("Rages".to_string(), rages.to_string()));
    }
    if let Some(rage_damage) = &class_specific_fields.rage_damage_bonus {
        vec.push(("Rage Damage".to_string(), rage_damage.to_string()));
    }

    // druid
    if let Some(cr) = &class_specific_fields.wild_shape_max_cr {
        vec.push(("Max. CR".to_string(), cr.to_string()));
    }

    // monk
    if let Some(ki) = &class_specific_fields.ki_points {
        vec.push(("Ki Points".to_string(), ki.to_string()));
    }
    if let Some(unarmored_movement) = &class_specific_fields.unarmored_movement {
        vec.push((
            "Unarmored Movement".to_string(),
            unarmored_movement.to_string(),
        ));
    }
    if let Some(martial_arts) = &class_specific_fields.martial_arts {
        vec.push((
            "Martial Arts".to_string(),
            format!("{}d{}", martial_arts.dice_value, martial_arts.dice_count),
        ));
    }

    // rogue
    if let Some(sneak) = &class_specific_fields.sneak_attack {
        vec.push((
            "Sneak Attack".to_string(),
            format!("{}d{}", sneak.dice_count, sneak.dice_value),
        ));
    }

    // sorcerer
    if let Some(sorcery_points) = class_specific_fields.sorcery_points {
        vec.push(("Sorcery Points".to_string(), sorcery_points.to_string()));
    }

    // warlock
    if let Some(invocations) = class_specific_fields.invocations_known {
        vec.push(("Invocations Known".to_string(), invocations.to_string()));
    }

    // wizard
    if let Some(arcane_recovery) = class_specific_fields.arcane_recovery_levels {
        vec.push(("Arcane Recovery".to_string(), arcane_recovery.to_string()));
    }

    vec
}

fn transpose(
    name: &str,
    input: [Vec<(String, String)>; 20],
) -> Result<Vec<(String, [String; 20])>, ClassError> {
    let mut output: HashMap<String, [Option<String>; 20]> = HashMap::new();

    for (i, column) in input.into_iter().enumerate() {
        for (key, value) in column {
            let entry = output
                .entry(key)
                .or_insert_with(|| std::array::from_fn(|_| None));

            entry[i] = Some(value);
        }
    }

    output
        .into_iter()
        .map(|(key, values)| {
            let values: Result<Vec<String>, _> = values
                .into_iter()
                .map(|value| {
                    value.ok_or(ClassError::ParseError(
                        format!("class specific field {} only partially complete", key),
                        name.to_string(),
                    ))
                })
                .collect();

            let values: [String; 20] = values?
                .try_into()
                .map_err(|_| ClassError::parse_error("invalid class specific arr len", name))?;
            Ok((key, values))
        })
        .collect()
}
