use crate::{
    character::{
        features::{AbilityScoreIncrease, FeatureEffect},
        stats::{Modifiers, Size, SkillModifiers, SkillType, StatType, Stats}, 
        Character
    }, 
    get::{get_background, get_class, get_race}
};

#[tokio::test]
async fn level_3_elf_monk() {
    let elf_future = get_race("elf");
    let monk_future = get_class("monk");
    let acolyte_future = get_background("acolyte");

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
    let halfling_future = get_race("halfling");
    let rogue_future = get_class("rogue");
    let acolyte_future = get_background("acolyte");

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
}
