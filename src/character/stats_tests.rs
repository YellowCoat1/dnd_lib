use super::stats::*;

#[test]
fn shorthands() {
    assert_eq!(StatType::Strength.get_shorthand().to_lowercase(), "str");
    assert_eq!(StatType::Constitution.get_shorthand().to_lowercase(), "con");
    assert_eq!(StatType::Charisma.get_shorthand().to_lowercase(), "cha");
}

#[test]
fn modifiers() {
    // setting strength to 20, wisdom to 12, and charisam to 14
    let stats = Stats::from_arr(&[20, 10, 10, 10, 12, 14]);

    let proficiency_bonus = 2;
    let modifiers = stats.modifiers();

    // making sure the base modifier calculation is correct.
    assert_eq!(modifiers.strength, 5, "Incorrect calculated strength modifier");
    assert_eq!(modifiers.wisdom, 1, "Incorrect calculated wisdom modifier");
    assert_eq!(modifiers.charisma, 2, "Incorrect calculated charisma modifier");
    assert_eq!(modifiers.dexterity, 0, "Incorrect calculated dexterity modifier");

    // getting the skill modifiers, with a proficiency in Insight and a proficiency bonus of 2.
    let mut skill_proficiencies = SkillProficiencies::default();
    skill_proficiencies.add_proficiency_from_type(SkillType::Insight);
    let skill_modifiers = skill_proficiencies.modifiers(&stats, proficiency_bonus);

    // Both of these are based on Wisdom, Wisdom's modifier is 1, and there's proficiency in insight 
    // so insight should be 3, and perception should be 1
    assert_eq!(skill_modifiers.insight, 3);
    assert_eq!(skill_modifiers.perception, 1);

    // testing saving throws
    let mut saves = Saves::default();
    saves.add_proficiency_from_type(StatType::Charisma);
    let save_modifiers = saves.modifiers(&stats, proficiency_bonus);
    
    assert_eq!(save_modifiers.charisma, 4, "Incorrect calculated charisma saving throw");
}

#[test]
fn add_stats() {
    let stats = Stats::from_arr(&[20, 10, 10, 10, 12, 14]);

    let subbed_stats = stats.clone() - 2;
    assert_eq!(subbed_stats, Stats::from_arr(&[18, 8, 8, 8, 10, 12]));
    
    let special_added_stats = stats.clone() + Stats::from_arr(&[0, 0, 0, 2, 2, 0]);
    assert_eq!(special_added_stats, Stats::from_arr(&[20, 10, 10, 12, 14, 14]));
    
    let mut grabbed_field_stats = stats.clone();
    *grabbed_field_stats.get_stat_type_mut(&StatType::Constitution) = 16;
    assert_eq!(grabbed_field_stats.constitution, 16);
}
