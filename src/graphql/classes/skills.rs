use crate::{graphql::classes::ClassError, rules2014::stats::SkillType};

use super::class_query::{ClassQueryClass, ClassQueryClassProficiencyChoicesFromOptionsItem};

pub fn process_skills(
    response_data: &ClassQueryClass,
) -> Result<(usize, Vec<SkillType>), ClassError> {
    let name = &response_data.name;
    let initial_proficiency_choice = response_data
        .proficiency_choices
        .first()
        .ok_or(ClassError::parse_error("intial proficiency choice", name))?;
    let choices: usize = initial_proficiency_choice
        .choose
        .try_into()
        .map_err(|_| ClassError::parse_error("intial proficiency choice parsing", name))?;
    let v = initial_proficiency_choice
        .from
        .options
        .iter()
        .map(|v| match &v.item {
            ClassQueryClassProficiencyChoicesFromOptionsItem::Proficiency(p) => Ok(&p.name),
            ClassQueryClassProficiencyChoicesFromOptionsItem::ProficiencyChoice => Err(
                ClassError::parse_error("unsupported skill prof choice", name),
            ),
        })
        .map(|v| {
            v.and_then(|z| {
                let skill_string = z
                    .get(7..)
                    .ok_or(ClassError::parse_error("skill string irregular", name))?;
                SkillType::from_name(skill_string)
                    .ok_or(ClassError::parse_error("skill string parse error", name))
            })
        })
        .collect::<Result<Vec<SkillType>, ClassError>>()?;
    Ok((choices, v))
}
