use futures::future::try_join_all;

use crate::{
    character::{
        Character, features::{AbilityScoreIncrease, FeatureEffect}, spells::{CASTER_SLOTS, PactSlots, SpellSlots}, stats::{Modifiers, Size, SkillModifiers, SkillType, StatType, Stats}
    }, 
    getter::DataProvider,
};

use crate::provider;

#[tokio::test]
async fn level_3_elf_monk() {
    let provider = provider();
    let elf_future = provider.get_race("elf");
    let monk_future = provider.get_class("monk");
    let acolyte_future = provider.get_background("acolyte");

    let elf = elf_future.await.expect("couldn't get human");
    let monk = monk_future.await.expect("couldnt't get monk");
    let acolyte = acolyte_future.await.expect("couldn't get acolyte");

    // Chosen using standard array
    let stats = Stats {
        strength: 10,
        dexterity: 14,
        constitution: 13,
        intelligence: 12,
        wisdom: 15,
        charisma: 8,
    };

    // georg is level 1
    let mut georg = Character::new("gerog".to_string(), &monk, &acolyte, &elf, stats);

    // add class items
    let class_items = &mut georg.classes.get_mut(0).expect("character should have a class").items;
    class_items[0].choose_in_place(0);
    class_items[1].choose_in_place(0);
    georg.add_class_items();
    // equip the shortsword
    georg.items[3].2 = true;

    assert_eq!(georg.items[0].0.name, "Clothes, common");
    assert_eq!(georg.items[3].0.name, "Shortsword");
    assert_eq!(georg.items[4].0.name, "Dungeoneer's Pack");


    // Choosing the skills we want
    georg.class_skill_proficiencies[0].choose_in_place(0);
    georg.class_skill_proficiencies[1].choose_in_place(1);

    // double checking they have exactly the skills we select, and the skill proficiencies granted
    // by the background
    let skills = georg.skills();
    let s_with_prof = skills.skills_with_proficiency().iter().map(|v| v.0).collect::<Vec<_>>();
    assert!(s_with_prof.contains(&SkillType::Acrobatics), "Character did not have a proficiency in Acrobatics");
    assert!(s_with_prof.contains(&SkillType::Athletics), "Character did not have a proficiency in Athletics");
    assert!(s_with_prof.contains(&SkillType::Insight), "Character did not have a proficiency in Insight");
    assert!(s_with_prof.contains(&SkillType::Religion), "Character did not ahve a proficiency in Religion");

    // choosing the subrace
    // there's only one option, (high elf) so we just choose the one available
    georg.race.subraces.choose_in_place(0);


    // level georg to level 3
    georg.level_up_to_level(&monk, 3);
    assert_eq!(georg.level(), 3);

    // double checking stats
    let final_stats = Stats {
        strength: 10,
        dexterity: 16,
        constitution: 13,
        intelligence: 13,
        wisdom: 15,
        charisma: 8,
    };
    assert_eq!(georg.stats(), final_stats);

    // double checking skills
    let skills = georg.skill_modifiers();
    let correct_skills = SkillModifiers {
        acrobatics: 5,
        animal_handling: 2,
        arcana: 1,
        athletics: 2,
        deception: -1,
        history: 1,
        insight: 4,
        intimidation: -1,
        investigation: 1,
        medicine: 2,
        nature: 1,
        perception: 2,
        performance: -1,
        persuasion: -1,
        religion: 3,
        sleight_of_hand: 3,
        stealth: 3,
        survival: 2,
    };

    assert_eq!(skills, correct_skills);


    let ac = georg.ac();
    assert_eq!(ac, 15);
}

#[tokio::test]
async fn level_5_halfling_rogue() {
    let provider = provider();
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
    let class_items = &mut bingus.classes.get_mut(0).expect("Halfing should have a class!").items;
    // Choosing rapier
    class_items.get_mut(0).expect("Halfling rogue should have an item choice").choose_in_place(0);
    // Choosing shortbow
    class_items.get_mut(1).expect("Halfling rogue should have a 2nd item choice").choose_in_place(0);
    // Choosing the pack to use (Dungeoneer's)
    class_items.get_mut(2).expect("Halfling rogue should have a 3nd item choice").choose_in_place(1);
    
    // add the items chosen
    bingus.add_class_items();

    // 6th item should be a Rapier
    assert_eq!(bingus.items.get(5).map(|v| v.0.name.clone()), Some("Rapier".to_string()));
    // 7th item should be a shortbow
    assert_eq!(bingus.items.get(6).map(|v| v.0.name.clone()), Some("Shortbow".to_string()));

    bingus.items[2].2 = true;


    // choose skill proficiencies granted by the class
    bingus.class_skill_proficiencies
        .get_mut(0)
        .expect("rogue should have skill proficiencies")
        .choose_in_place(8);
    bingus.class_skill_proficiencies
        .get_mut(1)
        .expect("rogue should have a 2nd skill proficiency")
        .choose_in_place(10);
    bingus.class_skill_proficiencies
        .get_mut(2)
        .expect("rogue should have a 3rd skill proficiency")
        .choose_in_place(2);
    bingus.class_skill_proficiencies
        .get_mut(3)
        .expect("rogue should have a 4th skill proficiency")
        .choose_in_place(4);

    // choosing the subrace
    bingus.race.subraces.choose_in_place(0);

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
    *ability_score_effect = AbilityScoreIncrease::StatIncrease(Some(StatType::Dexterity), Some(StatType::Constitution));


    // Now, with all of that out of the way, we check the skills and ability scores.
    
    let stats = bingus.stats();
    assert_eq!(stats, Stats::from(&[10, 18, 15, 12, 13, 9]));

    let skills = bingus.skill_modifiers();
    assert_eq!(skills, SkillModifiers {
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
    });

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
    assert_eq!(saves, Modifiers{stats: Stats::from(&[0, 7, 2, 4, 1, -1])});

    // Equipment proficiencies
    let equipment_proficiencies = bingus.equipment_proficiencies();
    assert!(equipment_proficiencies.simple_weapons);
    assert!(!equipment_proficiencies.martial_weapons);
    assert!(equipment_proficiencies.light_armor);
    assert!(!equipment_proficiencies.medium_armor);

    let mut other_proficiencies: Vec<_> = equipment_proficiencies.other
        .into_iter()
        .map(|v| v.to_lowercase())
        .collect();
    other_proficiencies.sort();
    assert_eq!(other_proficiencies, vec!["hand crossbows".to_string(), "longswords".to_string(), "rapiers".to_string(), "shortswords".to_string(), "thieves' tools".to_string()]);

    bingus.damage(30);
    assert_eq!(bingus.hp, 8, "Character had not taken damage properly");

    bingus.short_rest(0, None);
    assert_eq!(bingus.hp, 8, "Character healed from short rest when they should not have.");
    bingus.short_rest(1, None);
    assert_eq!(bingus.hp, 15, "Character did not heal the correct amount");
    bingus.short_rest(1, Some(vec![2]));
    assert_eq!(bingus.hp, 19, "Character did not heal the correct amount on manually inputed rolls");
    assert_eq!(bingus.spent_hit_dice, 2, "Incorrect amount of spent hit dice");
    bingus.long_rest();
    assert_eq!(bingus.hp, 38, "Character did not heal to full health on long rest");
    assert_eq!(bingus.spent_hit_dice, 0, "Character did not regain correct hit dice");
}

// testing a level 3 druid with spellcasting
#[tokio::test]
async fn level_3_druid() {
    let provider = provider();
    let human_future = provider.get_race("human");
    let druid_future = provider.get_class("druid");
    let acolyte_future = provider.get_background("acolyte");

    let spells = vec![
        // cantrips
        provider.get_spell("poison spray"),
        provider.get_spell("shillelagh"),
        // 1st level
        provider.get_spell("charm person"),
        provider.get_spell("cure wounds"),
        provider.get_spell("thunderwave"),
        provider.get_spell("healing word"),
        // 2nd level
        provider.get_spell("moonbeam"),
        provider.get_spell("darkvision"),
    ];


    let human = human_future .await.expect("couldn't get human");
    let druid = druid_future .await.expect("couldnt't get druid");
    let acolyte = acolyte_future .await.expect("couldn't get acolyte");


    let stats = Stats {
        strength: 8,
        constitution: 13,
        dexterity: 14,
        intelligence: 12, 
        wisdom: 15,
        charisma: 10,
    };

    let mut boopo = Character::new("Boopo".to_string(), &druid, &acolyte, &human, stats);

    // choose skill proficiencies granted by the class 
    
    boopo.class_skill_proficiencies
        .get_mut(0)
        .expect("Character should have a 1st choice for skill proficiencies")
        // this is the 6th choice, which is perception
        .choose_in_place(5);

    boopo.class_skill_proficiencies
        .get_mut(1)
        .expect("Character should have a 2nd choice for skill proficiencies")
        // this is the 8th choice, which is Survival
        .choose_in_place(7);

    boopo.level_up_to_level(&druid, 3);
    
    // choose subclass
    boopo.classes[0].subclass.choose_in_place(0);

    assert_eq!(boopo.spell_slots(), Some(SpellSlots(CASTER_SLOTS[2])));
    assert_eq!(boopo.available_spell_slots, Some(SpellSlots(CASTER_SLOTS[2])));
    assert_eq!(boopo.available_pact_slots, None);

    let v = boopo.prepare_spells();
    assert_eq!(v.len(), 1, "There were more classes returned by the spells prepared utility than there should be");
    let (_, prepped_spell_list, amount, cantrips) = v.into_iter().next().unwrap();
    assert_eq!(amount, 6, "incorrect number of spells to prepare");
    assert_eq!(cantrips, 2, "incorrect number of cantrips to prepare");
    *prepped_spell_list = try_join_all(spells).await
        .expect("Couldn't get spells");

    let spells = boopo.classes[0].spellcasting
        .as_ref()
        .expect("Druid should be a spellcaster")
        .1
        .iter()
        .map(|spell| spell.name.to_lowercase())
        .collect::<Vec<_>>();
    let spells_lower = spells.iter().map(|v| v.to_lowercase()).collect::<Vec<_>>();
    assert_eq!(spells_lower, vec!["poison spray", "shillelagh", "charm person", "cure wounds", "thunderwave", "healing word", "moonbeam", "darkvision"]);


    let spell_attacks = boopo.spell_actions();
    // macro to help test spell attack damage
    macro_rules! assert_spell_damage {
        ($char:expr, $spell_index:expr, $expected_damage:expr) => {
            let spell_attack = spell_attacks.get($spell_index);
            match spell_attack {
                Some(attack) => {
                    assert_eq!(attack.damage_roll.to_string(), $expected_damage, "Damage roll for spell at index '{}' did not match expected value", $spell_index);
                },
                None => panic!("Spell attack at index '{}' not found", $spell_index),
            }
        };
    }

    assert_spell_damage!(boopo, 0, "1d12 Poison"); // Poison Spray
    assert_spell_damage!(boopo, 1, "2d8 Thunder"); // Thunderwave (1st level)
    assert_spell_damage!(boopo, 2, "3d8 Thunder"); // Thunderwave (2nd level)
    assert_spell_damage!(boopo, 3, "2d10 Radiant"); // Moonbeam


    let poison_spray = spell_attacks.first().expect("Couldn't get poison spray spell attack");
    let moonbeam = spell_attacks.get(3).expect("Couldn't get moonbeam spell attack");

    boopo.cast(poison_spray, None);
    assert_eq!(boopo.available_spell_slots, Some(SpellSlots([4, 2, 0, 0, 0, 0, 0, 0, 0])), "Spell slots after casting poison spray did not match expected value");
    boopo.cast(moonbeam, None);
    assert_eq!(boopo.available_spell_slots, Some(SpellSlots([4, 1, 0, 0, 0, 0, 0, 0, 0])), "Spell slots after casting moonbeam did not match expected value");

    boopo.long_rest();
    assert_eq!(boopo.available_spell_slots, Some(SpellSlots([4, 2, 0, 0, 0, 0, 0, 0, 0])), "Spell slots after long rest did not match expected value");
}

#[tokio::test]
async fn level_10_warlock() {
    let provider = provider();
    let tiefling_future = provider.get_race("tiefling");
    let warlock_future = provider.get_class("warlock");
    let acolyte_future = provider.get_background("acolyte");


    let spells = vec![
        // cantrips
        provider.get_spell("eldritch blast"),
        provider.get_spell("minor illusion"),
        // 1st level
        provider.get_spell("hellish rebuke"),
        provider.get_spell("charm person"),
        // 2nd level
        provider.get_spell("darkness"),
        provider.get_spell("invisibility"),
        // 3rd level
        provider.get_spell("counterspell"),
        provider.get_spell("dispel magic"),
        // 4th level
        provider.get_spell("blight"),
        provider.get_spell("banishment"),
        // 5th level
        provider.get_spell("hold monster"),
        provider.get_spell("dream"),
    ];


    let tiefling = tiefling_future.await.expect("couldn't get tiefling");
    let warlock = warlock_future.await.expect("couldnt't get warlock");
    let acolyte = acolyte_future.await.expect("couldn't get acolyte");

    let stats = Stats {
        strength: 8,
        constitution: 14,
        dexterity: 13,
        intelligence: 12,
        wisdom: 10,
        charisma: 15,
    };

    let mut baroopa = Character::new("Baroopa".to_string(), &warlock, &acolyte, &tiefling, stats);

    // choose skill proficiencies granted by the class

    // deception
    baroopa.class_skill_proficiencies
        .get_mut(0)
        .expect("Character should have a 1st choice for skill proficiencies")
        .choose_in_place(1);
    // Investigation
    baroopa.class_skill_proficiencies
        .get_mut(1)
        .expect("Character should have a 2nd choice for skill proficiencies")
        .choose_in_place(4);


    baroopa.level_up_to_level(&warlock, 10); 

    // choose subclass
    // this is the fiend patron
    baroopa.classes[0].subclass.choose_in_place(0);


    // find all ability score increases
    let ability_score_increases = baroopa.classes[0].current_class_features
        .iter_mut()
        .flat_map( |level_features|level_features.iter_mut())
        .filter_map(|v|  v.as_base_mut())
        .flat_map(|v| v.effects.iter_mut())
        .filter_map(|v| match v {
            FeatureEffect::AbilityScoreIncrease(a) => Some(a),
            _ => None,
        });


    let mut ability_score_increases_vec = ability_score_increases.collect::<Vec<_>>();
    assert_eq!(ability_score_increases_vec.len(), 2, "Warlock should have 2 ability score increases by level 10");
    ability_score_increases_vec[0]
        .set_stat_increase(StatType::Charisma, Some(StatType::Charisma));
    ability_score_increases_vec[1]
        .set_stat_increase(StatType::Charisma, Some(StatType::Dexterity));

    assert_eq!(baroopa.stats(), Stats::from(&[8, 14, 14, 13, 10, 20]));

    // There should be no spells to prepare for warlock, as they know their spells.
    assert_eq!(baroopa.prepare_spells().len(), 0);

    // add the spells
    let warlock_spells = &mut baroopa.classes[0].spellcasting
        .as_mut()
        .expect("Warlock should be a spellcaster")
       .1;
    *warlock_spells = try_join_all(spells).await
        .expect("Couldn't get spells");


    let warlock_spells = baroopa.classes[0].spellcasting
        .as_ref()
        .expect("Warlock should be a spellcaster")
        .1
        .iter()
        .map(|spell| spell.name.to_lowercase())
        .collect::<Vec<_>>();

    assert_eq!(warlock_spells, vec![
        "eldritch blast",
        "minor illusion",
        "hellish rebuke",
        "charm person",
        "darkness",
        "invisibility",
        "counterspell",
        "dispel magic",
        "blight",
        "banishment",
        "hold monster",
        "dream",
    ]);

    assert_eq!(baroopa.classes[0].spellcasting.as_ref().unwrap().1.len(), 12, "Warlock should have 12 spells known at level 10");

    let spell_actions = baroopa.spell_actions();
    assert_eq!(spell_actions.len(), 9, "Warlock should have 9 spell actions at level 10");
    assert_eq!(spell_actions[0].name.to_lowercase(), "eldritch blast", "First spell action should be eldritch blast");
    assert_eq!(spell_actions[0].damage_roll.to_string(), "1d10 Force", "Eldritch blast damage roll should be 1d10 Force");
    
    assert_eq!(spell_actions[1].name.to_lowercase(), "hellish rebuke", "2nd spell action should be hellish rebuke");
    assert_eq!(spell_actions[1].damage_roll.to_string(), "2d10 Fire", "Hellish rebuke damage roll should be 2d10 Fire");
    assert_eq!(spell_actions[2].name.to_lowercase(), "hellish rebuke", "3nd spell action should be hellish rebuke");
    assert_eq!(spell_actions[2].damage_roll.to_string(), "3d10 Fire", "Hellish rebuke damage roll should be 3d10 Fire");
    

    // pact slots should be 2 at level 10, and level 5
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 2, level: 5 }), "Pact slots at level 10 did not match expected value");

    // cast eldritch blast 3 times
    for _ in 0..3 {
        let eldritch_blast = spell_actions.first().expect("Couldn't get eldritch blast spell attack");
        baroopa.cast(eldritch_blast, None);
    }
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 2, level: 5 }), "Pact slots after casting eldritch blast 3 times did not match expected value");

    // Casting hellish rebuke at 5th level
    let hellish_rebuke = spell_actions.get(5).expect("Couldn't get hellish rebuke spell attack");
    assert_eq!(hellish_rebuke.damage_roll.to_string(), "6d10 Fire", "Hellish rebuke damage roll when cast at 5th level did not match expected value");
    baroopa.cast(hellish_rebuke, None);
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 1, level: 5 }), "Pact slots after casting hellish rebuke did not match expected value");

    // Casting blight at 5th level
    let blight = spell_actions.get(7).expect("Couldn't get blight spell attack");
    assert_eq!(blight.damage_roll.to_string(), "9d8 Necrotic", "Blight damage roll when cast at 5th level did not match expected value");
    baroopa.cast(blight, None);
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 0, level: 5 }), "Pact slots after casting blight did not match expected value");

    // short rest should restore pact slots
    baroopa.short_rest(0, None);
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 2, level: 5 }), "Pact slots after short rest did not match expected value");

    // long rest should restore pact slots
    baroopa.cast(hellish_rebuke, None);
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 1, level: 5 }), "Pact slots after casting hellish rebuke did not match expected value");
    baroopa.long_rest();
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 2, level: 5 }), "Pact slots after long rest did not match expected value");
}
