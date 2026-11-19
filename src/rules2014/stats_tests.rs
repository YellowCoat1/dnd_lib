use std::{collections::HashSet, str::FromStr};
use strum::IntoEnumIterator;

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
    let stats = Stats::from(&[20, 10, 10, 10, 12, 14]);

    let proficiency_bonus = 2;
    let modifiers = stats.modifiers();

    // making sure the base modifier calculation is correct.
    assert_eq!(
        modifiers.stats.strength, 5,
        "Incorrect calculated strength modifier"
    );
    assert_eq!(
        modifiers.stats.wisdom, 1,
        "Incorrect calculated wisdom modifier"
    );
    assert_eq!(
        modifiers.stats.charisma, 2,
        "Incorrect calculated charisma modifier"
    );
    assert_eq!(
        modifiers.stats.dexterity, 0,
        "Incorrect calculated dexterity modifier"
    );

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

    assert_eq!(
        save_modifiers.stats.charisma, 4,
        "Incorrect calculated charisma saving throw"
    );

    assert_eq!(
        Modifiers::default(),
        Modifiers {
            stats: Stats::from(&[0, 0, 0, 0, 0, 0])
        }
    );
}

#[test]
fn add_stats() {
    let mut stats = Stats::from(&[20, 10, 10, 10, 12, 14]);

    let subbed_stats = stats - 2;
    assert_eq!(subbed_stats, Stats::from(&[18, 8, 8, 8, 10, 12]));

    let special_added_stats = stats + Stats::from(&[0, 0, 0, 2, 2, 0]);
    assert_eq!(special_added_stats, Stats::from(&[20, 10, 10, 12, 14, 14]));

    *stats.get_stat_type_mut(&StatType::Constitution) = 16;
    assert_eq!(stats.constitution, 16);

    let other_stats = Stats::from(&[1, 3, 2, 1, 2, 2]);
    stats = stats - other_stats;
    assert_eq!(stats, Stats::from(&[19, 7, 14, 9, 10, 12]));

    assert_eq!(
        <Stats as Into<Vec<_>>>::into(other_stats),
        vec![1, 3, 2, 1, 2, 2]
    );
}

#[test]
fn equipment() {
    let equipment1 = EquipmentProficiencies {
        simple_weapons: true,
        light_armor: true,
        ..EquipmentProficiencies::default()
    };
    let added_equipment = EquipmentProficiencies {
        heavy_armor: true,
        other: HashSet::from([String::from("dragonchess")]),
        ..EquipmentProficiencies::default()
    };

    let final_equipment = equipment1 + added_equipment;

    assert_eq!(
        final_equipment,
        EquipmentProficiencies {
            simple_weapons: true,
            martial_weapons: false,
            light_armor: true,
            medium_armor: false,
            heavy_armor: true,
            shields: false,
            other: HashSet::from([String::from("dragonchess")]),
        }
    );
}

#[test]
fn stat_list_conversion() {
    let saves_list = Saves::from_stats(StatType::iter().collect::<Vec<_>>());

    for stat in StatType::iter() {
        assert!(saves_list.is_proficient(stat))
    }

    let mut other_saves_list = Saves::default();

    for stat in StatType::iter() {
        other_saves_list.add_proficiency_from_type(stat);
    }

    assert_eq!(saves_list, other_saves_list);

    for stat in StatType::iter() {
        assert!(StatType::from_str(&stat.to_string()).is_ok())
    }
}

#[test]
fn skill_modifications() {
    let mut modifiers = SkillModifiers::default();
    for skill in SkillType::iter() {
        modifiers[skill] += 10;
    }
    for skill in SkillType::iter() {
        assert_eq!(modifiers[skill], 10);
    }

    let mut proficiencies = SkillProficiencies::default();
    for skill in SkillType::iter() {
        proficiencies[skill].proficiency = true
    }
    for skill in SkillType::iter() {
        assert!(proficiencies[skill].proficiency)
    }
}

#[test]
fn stats_addition() {
    let left = Stats {
        strength: 1,
        dexterity: 2,
        constitution: 3,
        wisdom: 4,
        intelligence: 5,
        charisma: 6,
    };
    let right = Stats {
        strength: 6,
        dexterity: 5,
        constitution: 4,
        wisdom: 3,
        intelligence: 2,
        charisma: 1,
    };

    assert_eq!(
        left + right,
        Stats {
            strength: 7,
            dexterity: 7,
            constitution: 7,
            wisdom: 7,
            intelligence: 7,
            charisma: 7,
        }
    );
}

#[test]
fn modifiers_as_ref_and_as_mut_preserve_their_values() {
    let mut modifiers = Modifiers {
        stats: Stats {
            strength: 1,
            dexterity: 2,
            constitution: 3,
            wisdom: 4,
            intelligence: 5,
            charisma: 6,
        },
    };

    assert_eq!(
        modifiers.as_ref(),
        &Stats {
            strength: 1,
            dexterity: 2,
            constitution: 3,
            wisdom: 4,
            intelligence: 5,
            charisma: 6,
        }
    );

    modifiers.as_mut().strength = 9;
    assert_eq!(modifiers.stats.strength, 9);
}

#[test]
fn saves_modifiers_include_only_each_proficiency() {
    let stats = Stats {
        strength: 20,
        dexterity: 18,
        constitution: 16,
        wisdom: 14,
        intelligence: 12,
        charisma: 10,
    };
    let saves = Saves {
        constitution: true,
        wisdom: true,
        ..Saves::default()
    };

    let modifiers = saves.modifiers(&stats, 3);
    assert_eq!(modifiers.stats.constitution, 6);
    assert_eq!(modifiers.stats.wisdom, 5);
    assert_eq!(modifiers.stats.strength, 5);
    assert_eq!(modifiers.stats.dexterity, 4);
    assert_eq!(modifiers.stats.intelligence, 1);
    assert_eq!(modifiers.stats.charisma, 0);

    assert!(saves.is_proficient(StatType::Constitution));
    assert!(saves.is_proficient(StatType::Wisdom));
    assert!(!saves.is_proficient(StatType::Strength));
}

#[test]
fn proficiency_and_expertise_modifiers() {
    let stats = Stats {
        strength: 14,
        dexterity: 14,
        constitution: 14,
        wisdom: 14,
        intelligence: 14,
        charisma: 14,
    };
    let mut proficiencies = SkillProficiencies::default();
    for skill in SkillType::iter() {
        proficiencies.add_proficiency_from_type(skill);
        proficiencies.add_expertise_from_type(skill);
    }

    let modifiers = proficiencies.modifiers(&stats, 3);

    for skill in SkillType::iter() {
        assert_eq!(*modifiers.get_skill_type(skill), 8);
    }
}

#[test]
fn equipment_proficiency_unions() {
    let left_only = EquipmentProficiencies {
        martial_weapons: true,
        medium_armor: true,
        heavy_armor: true,
        shields: true,
        ..EquipmentProficiencies::default()
    };
    let right_only = EquipmentProficiencies {
        martial_weapons: false,
        medium_armor: false,
        heavy_armor: false,
        shields: false,
        ..EquipmentProficiencies::default()
    };

    let added = left_only.clone() + right_only.clone();
    assert!(added.martial_weapons);
    assert!(added.medium_armor);
    assert!(added.heavy_armor);
    assert!(added.shields);

    let mut added_assign = right_only.clone();
    added_assign += left_only.clone();
    assert!(added_assign.martial_weapons);
    assert!(added_assign.medium_armor);
    assert!(added_assign.heavy_armor);
    assert!(added_assign.shields);

    let mut reverse_assign = left_only.clone();
    reverse_assign += right_only;
    assert!(reverse_assign.martial_weapons);
    assert!(reverse_assign.medium_armor);
    assert!(reverse_assign.heavy_armor);
    assert!(reverse_assign.shields);
}
