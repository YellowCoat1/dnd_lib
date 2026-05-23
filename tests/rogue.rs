#![cfg(feature = "network-intensive-tests")]
use dnd_lib::prelude::*;
use dnd_lib::rules2014::features::{Feature, FeatureEffect};
use dnd_lib::rules2014::items::{DamageRoll, DamageType};
use dnd_lib::rules2014::spells::SpellAction;
use dnd_lib::rules2014::stats::{Modifiers, Size, SkillModifiers, SkillType, StatType};

#[tokio::test]
async fn level_5_halfling_rogue() {
    let provider = Dnd5eapiGetter::new();
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

    // create the character
    let mut bingus = CharacterBuilder::new("bingus")
        .class(&rogue)
        .background(&acolyte)
        .race(&halfling)
        .stats(stats)
        .build()
        .expect("failed to build the rogue character");

    // check bingus's starting items
    assert_eq!(bingus.unchosen_items().len(), 3);
    assert_eq!(bingus.get_unchosen_categories().len(), 0);

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

    bingus
        .items
        .get_mut(2)
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
    let expertise = bingus.classes[0] // the first class,
        .current_class_features[0] // the features for 1st level
        .get_mut(0) // and the first such feature.
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

    // at 4th level there is also an ability score increase.

    // get the ability score increase
    let mut ability_score_increases = bingus.ability_score_increases_mut();
    let score_increase = ability_score_increases
        .get_mut(0)
        .expect("rogue should have an ability score increase");

    // set the score increase to dex and con
    score_increase.set_stat_increase(StatType::Dexterity, Some(StatType::Constitution));

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

    let acrobatics_bonus = Feature {
        name: String::new(),
        description: vec![],
        effects: vec![FeatureEffect::AddSkillModifier(SkillType::Acrobatics, 1)],
    };
    bingus.bonus_features.push(acrobatics_bonus);
    let updated_acrobatics_skills = bingus.skill_modifiers();
    assert_eq!(updated_acrobatics_skills.acrobatics, 5);

    let history_proficiency = Feature {
        name: String::new(),
        description: vec![],
        effects: vec![FeatureEffect::AddSkillProficiency(SkillType::History)],
    };
    bingus.bonus_features.push(history_proficiency);
    let updated_history_skills = bingus.skill_modifiers();
    assert_eq!(updated_history_skills.history, 4);

    // hp should be 38
    assert_eq!(bingus.max_hp(), 38, "rogue has wrong max hp");
    assert_eq!(bingus.hp, 38, "rogue is not at max hp after level-up");

    // bingus has leather armor on.
    // This grants 11+DEX, which here is 11+4.
    assert_eq!(bingus.ac(), 15, "rogue has the wrong ac");

    assert_eq!(
        bingus.descriptors.size,
        Size::Small,
        "rogue is not small, as a halfling should be"
    );

    // Testing saving throw modifiers
    let saves = bingus.save_mods();
    assert_eq!(
        saves,
        Modifiers {
            stats: Stats::from(&[0, 7, 2, 4, 1, -1])
        },
        "rogue has wrong saving throw modifiers"
    );
    // add a feature that adds proficiency in strength
    bingus.bonus_features.push(Feature {
        name: "Strength saving throw proficiency".to_string(),
        description: vec![],
        effects: vec![FeatureEffect::AddSaveProficiency(StatType::Strength)],
    });
    // Proficiency bonus is +3, so the strength modifier is now 0+3=3
    assert_eq!(
        bingus.save_mods(),
        Modifiers {
            stats: Stats::from(&[3, 7, 2, 4, 1, -1])
        },
        "rogue has wrong saving throw modifiers after adding strength save proficiency"
    );

    // then, add 1 to the save with another feature
    bingus.bonus_features.push(Feature {
        name: "Strength save bonus".to_string(),
        description: vec![],
        effects: vec![FeatureEffect::AddSaveModifier(StatType::Strength, 1)],
    });
    assert_eq!(
        bingus.save_mods(),
        Modifiers {
            stats: Stats::from(&[4, 7, 2, 4, 1, -1])
        },
        "rogue has wrong saving throw modifiers after adding strength save proficiency"
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

    // testing the rogue's different speeds
    let speeds = bingus.speeds();
    // check that there's only the default
    assert_eq!(speeds.walking, Some(25));
    assert_eq!(
        (
            speeds.flying,
            speeds.hovering,
            speeds.burrowing,
            speeds.climbing,
            speeds.swimming
        ),
        (None, None, None, None, None),
        "rogue should have no special speeds"
    );

    // add a swim speed
    let swimming_speed_feature = Feature {
        name: String::from("Swimmer"),
        description: vec![],
        effects: vec![FeatureEffect::SwimmingSpeed(30)],
    };
    bingus.bonus_features.push(swimming_speed_feature);
    // check that the swim speed had an effect
    assert_eq!(
        bingus.speeds().swimming,
        Some(30),
        "rogue should have a swim speed"
    );

    // make sure bingus cannot cast spells
    let spell = SpellAction {
        name: String::new(),
        spell_level: 1,
        damage_roll: DamageRoll::new(0, 0, 0, dnd_lib::rules2014::items::DamageType::Slashing),
        spell_attack_mod: 0,
    };

    let casting_result = bingus.cast(&spell, None);
    assert!(!casting_result, "rogue should not be able to cast spells");

    // Damage & Health

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

    // Weapon Actions

    // equip items
    bingus
        .items
        .get_mut(3)
        .expect("Rogue should have a dagger")
        .equip(); // dagger
    bingus
        .items
        .get_mut(5)
        .expect("Rogue should have a rapier")
        .equip(); // rapier
                  // see the actions that bingus can take with these weapons
    let mut weapon_actions = bingus.weapon_actions();
    assert_eq!(
        weapon_actions.len(),
        4,
        "Rogue should have 4 weapon actions: 2 for daggers, 1 for the rapier, and 1 for unarmed."
    );
    weapon_actions.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(weapon_actions[0].name, "Dagger");
    assert_eq!(weapon_actions[1].name, "Dagger");

    // one should have second attack, and one shouldn't.
    assert!(!weapon_actions[0].second_attack || !weapon_actions[1].second_attack);
    assert!(weapon_actions[0].second_attack || weapon_actions[1].second_attack);

    assert_eq!(weapon_actions[2].name, "Rapier");
    assert_eq!(weapon_actions[2].attack_bonus, 3 + 4);
    assert_eq!(
        weapon_actions[2].damage_roll,
        DamageRoll::new(1, 8, 4, DamageType::Piercing)
    );

    assert_eq!(weapon_actions[3].name, "Unarmed Strike");
    assert_eq!(
        weapon_actions[3].damage_roll,
        DamageRoll::new(0, 0, 1, DamageType::Bludgeoning)
    );
}
