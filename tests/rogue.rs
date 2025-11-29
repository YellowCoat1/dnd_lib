#![cfg(feature = "network-intensive-tests")]
use dnd_lib::character::features::{AbilityScoreIncrease, FeatureEffect};
use dnd_lib::character::stats::{Modifiers, Size, SkillModifiers, SkillType, StatType};
use dnd_lib::prelude::*;

#[tokio::test]
async fn level_5_halfling_rogue() {
    let provider = Dnd5eapigetter::new();
    let halfling_future = provider.get_race("halfling");
    let rogue_future = provider.get_class("rogue");
    let acolyte_future = provider.get_background("acolyte");

    let halfling = halfling_future.await.expect("couldn't get halfling");
    let rogue = rogue_future.await.expect("couldnt't get rogue");
    let acolyte = acolyte_future.await.expect("couldn't get acolyte");

    // Chosen using standard array
    let stats = Stats {
        strength: 10,
        dexterity: 15,
        constitution: 14,
        intelligence: 12,
        wisdom: 13,
        charisma: 8,
    };

    let mut bingus = Character::new("bingus".to_string(), &rogue, &acolyte, &halfling, stats);

    // add bingus's items
    bingus.choose_items(0, 0); // rapier
    bingus.choose_items(1, 0); // shortbow
    bingus.choose_items(2, 1); // dungeoneer's pack
    bingus.add_chosen_items();

    // 6th item should be a Rapier
    assert_eq!(
        bingus.items.get(5).map(|v| v.item.name.clone()),
        Some("Rapier".to_string())
    );
    // 7th item should be a shortbow
    assert_eq!(
        bingus.items.get(6).map(|v| v.item.name.clone()),
        Some("Shortbow".to_string())
    );

    bingus.items.get_mut(2)
        .expect("should have leather armor")
        .equip();

    // choose skill proficiencies granted by the class

    // Acrobatics
    bingus
        .class_skill_proficiencies
        .get_mut(0)
        .expect("rogue should have skill proficiencies")
        .choose_in_place(8);
    // Stealth
    bingus
        .class_skill_proficiencies
        .get_mut(1)
        .expect("rogue should have a 2nd skill proficiency")
        .choose_in_place(10);
    // Deception
    bingus
        .class_skill_proficiencies
        .get_mut(2)
        .expect("rogue should have a 3rd skill proficiency")
        .choose_in_place(2);
    // Intimidation
    bingus
        .class_skill_proficiencies
        .get_mut(3)
        .expect("rogue should have a 4th skill proficiency")
        .choose_in_place(4);

    // choosing the subrace
    bingus.race.choose_subrace(0); // lightfoot

    // level bingus up to level 5
    bingus.level_up_to_level(&rogue, 5);

    assert_eq!(bingus.level(), 5);

    // Proficiency bonus at level 5 is 3.
    assert_eq!(bingus.proficiency_bonus(), 3);

    // We want to get the first feature of rogue.
    // This is bingus's first class, and in the features of that class, the first level and the
    // first feature of that level.
    let expertise = bingus.classes[0].current_class_features[0]
        .get_mut(0)
        .expect("Rogue should have level 1 features")
        .as_base_mut()
        .expect("Rogue should have expertise");

    assert_eq!(expertise.name, String::from("Expertise"));

    // expertise has an effect which we want to manipulate. First we want to confirm it's there.
    let expertise_effect = match expertise.effects.get_mut(0) {
        Some(FeatureEffect::Expertise(o)) => o,
        _ => panic!("Expertise should have an expertise effect"),
    };

    // Then, we set the expertise to the skills we want.
    // There's no checks here that we're setting this to something we're already proficient in.
    // You'd need to check yourself that the user inputted SkillType is already proficient. If it
    // isn't, expertise just acts like proficiency.
    expertise_effect[0] = Some(SkillType::Deception);
    expertise_effect[1] = Some(SkillType::Stealth);

    // We also want to choose the subclass.
    bingus.classes[0].subclass.choose_in_place(0);

    // at 4th level there's also an ability score increase.

    // this massive line is imposing, but it's just fetching the specific feature we want to fill.
    let ability_score_increase = &mut bingus.classes[0].current_class_features[3]
        .get_mut(0)
        .expect("Rogue should have 4th level features")
        .as_base_mut()
        .expect("Rogue should have 4th level features")
        // here, we've gotten the specific Feature we want. the last 3 lines are to access the
        // FeatureEffect.
        .effects
        .get_mut(0)
        .expect("Rogue's 4th level feature should have an effect");

    // Now that we've gotten the FeatureEffect, we just match it to make sure it's an
    // AbilityScoreIncrease.
    let ability_score_effect = match ability_score_increase {
        FeatureEffect::AbilityScoreIncrease(a) => a,
        _ => panic!("Rogue's first 4th level feature should be an ability score increase"),
    };

    // finally, now that we have it, we just set it to a stat increase of dexterity and
    // consitituion
    *ability_score_effect =
        AbilityScoreIncrease::StatIncrease(Some(StatType::Dexterity), Some(StatType::Constitution));

    // Now, with all of that out of the way, we check the skills and ability scores.

    let stats = bingus.stats();
    assert_eq!(stats, Stats::from(&[10, 18, 15, 12, 13, 9]));

    let skills = bingus.skill_modifiers();
    assert_eq!(
        skills,
        SkillModifiers {
            acrobatics: 4,
            animal_handling: 1,
            arcana: 1,
            athletics: 0,
            deception: 5,
            history: 1,
            insight: 4,
            intimidation: 2,
            investigation: 1,
            medicine: 1,
            nature: 1,
            perception: 1,
            performance: -1,
            persuasion: 2,
            religion: 4,
            sleight_of_hand: 4,
            stealth: 10,
            survival: 1,
        }
    );

    // hp should be 38
    assert_eq!(bingus.max_hp(), 38);
    assert_eq!(bingus.hp, 38);

    // bingus has leather armor on.
    // This grants 11+DEX, which here is 11+4.
    assert_eq!(bingus.ac(), 15);

    assert_eq!(bingus.speed(), 25);

    assert_eq!(bingus.descriptors.size, Size::Small);

    // Testing saving throw modifiers
    let saves = bingus.save_mods();
    assert_eq!(
        saves,
        Modifiers {
            stats: Stats::from(&[0, 7, 2, 4, 1, -1])
        }
    );

    // Equipment proficiencies
    let equipment_proficiencies = bingus.equipment_proficiencies();
    assert!(equipment_proficiencies.simple_weapons);
    assert!(!equipment_proficiencies.martial_weapons);
    assert!(equipment_proficiencies.light_armor);
    assert!(!equipment_proficiencies.medium_armor);

    let mut other_proficiencies: Vec<_> = equipment_proficiencies
        .other
        .into_iter()
        .map(|v| v.to_lowercase())
        .collect();
    other_proficiencies.sort();
    assert_eq!(
        other_proficiencies,
        vec![
            "hand crossbows".to_string(),
            "longswords".to_string(),
            "rapiers".to_string(),
            "shortswords".to_string(),
            "thieves' tools".to_string()
        ]
    );

    bingus.damage(30);
    assert_eq!(bingus.hp, 8, "Character had not taken damage properly");

    bingus.short_rest(0, None);
    assert_eq!(
        bingus.hp, 8,
        "Character healed from short rest when they should not have."
    );
    bingus.short_rest(1, None);
    assert_eq!(bingus.hp, 15, "Character did not heal the correct amount");
    bingus.short_rest(1, Some(vec![2]));
    assert_eq!(
        bingus.hp, 19,
        "Character did not heal the correct amount on manually inputed rolls"
    );
    assert_eq!(
        bingus.spent_hit_dice, 2,
        "Incorrect amount of spent hit dice"
    );
    bingus.long_rest();
    assert_eq!(
        bingus.hp, 38,
        "Character did not heal to full health on long rest"
    );
    assert_eq!(
        bingus.spent_hit_dice, 0,
        "Character did not regain correct hit dice"
    );
}
