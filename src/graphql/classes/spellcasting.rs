use std::str::FromStr;

use crate::{
    graphql::classes::{class_query::ClassQueryClass, ClassError},
    rules2014::{
        spells::{SpellCasterType, SpellCastingPreperation, Spellcasting},
        stats::StatType,
    },
};

pub fn process_spellcasting(
    response_data: &ClassQueryClass,
) -> Result<Option<Spellcasting>, ClassError> {
    let name = &response_data.name;
    let spellcasting = match response_data.spellcasting.as_ref() {
        Some(s) => s,
        None => return Ok(None),
    };
    let spellcasting_levels = match response_data
        .class_levels
        .iter()
        .map(|z| z.spellcasting.as_ref())
        .collect::<Option<Vec<_>>>()
    {
        Some(s) => s,
        None => return Ok(None),
    };

    let spellcaster_type = match spellcasting.level {
        _ if name.trim().to_lowercase() == "warlock" => SpellCasterType::Warlock,
        1 => SpellCasterType::Full,
        2 => SpellCasterType::Half,
        3 => SpellCasterType::Quarter,
        o => {
            return Err(ClassError::parse_error(
                format!("unrecognized spellcaster level {o}").as_str(),
                name,
            ))
        }
    };

    let spellcasting_ability = StatType::from_str(&spellcasting.spellcasting_ability.name)
        .map_err(|_| ClassError::parse_error("spellcasting ability stat type", name))?;
    let spells_known_first_level = spellcasting_levels
        .first()
        .ok_or(ClassError::parse_error(
            "spellcasting levels no first level",
            name,
        ))?
        .spells_known;
    let preperation_type = match spells_known_first_level {
        Some(_) => SpellCastingPreperation::Known,
        None => SpellCastingPreperation::Prepared,
    };

    let cantrips_per_level: [usize; 20] = spellcasting_levels
        .iter()
        .map(|z| {
            z.cantrips_known
                .ok_or(ClassError::parse_error("no cantrips", name))
        })
        .map(|y| {
            y.and_then(|z| {
                z.try_into()
                    .map_err(|_| ClassError::parse_error("cantrip count parsing", name))
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| ClassError::parse_error("leveled cantrips != 20 levels", name))?;

    let spells: Vec<(&String, usize)> = response_data
        .spells
        .iter()
        .map(|v| {
            v.level
                .try_into()
                .map(|z| (&v.name, z))
                .map_err(|_| ClassError::parse_error("spell level", name))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut spell_list: [Vec<String>; 10] = Default::default();
    for (spell, level) in spells {
        spell_list
            .get_mut(level)
            .ok_or(ClassError::parse_error("spell level out of range", name))?
            .push(spell.clone());
    }
    Ok(Some(Spellcasting {
        spell_list,
        spellcasting_ability,
        spellcaster_type,
        preperation_type,
        cantrips_per_level,
    }))
}
