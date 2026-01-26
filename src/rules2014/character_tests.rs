#![cfg(feature = "network-intensive-tests")]
use super::player_character::Character;
use super::stats::Stats;
use super::{
    class::TrackedField,
    features::{Feature, FeatureEffect},
    stats::StatType,
};
use crate::getter::DataProvider;

use crate::{provider, CharacterBuilder};

#[tokio::test]
async fn char_stats() {
    let provider = provider();
    let wizard_future = provider.get_class("wizard");
    let acolyte_future = provider.get_background("acolyte");
    let human_future = provider.get_race("human");
    let dwarf_future = provider.get_race("dwarf");

    let (wizard, acolyte) = (wizard_future.await.unwrap(), acolyte_future.await.unwrap());
    let (human, dwarf) = (human_future.await.unwrap(), dwarf_future.await.unwrap());

    let john = Character::new(
        String::from("john"),
        &wizard,
        &acolyte,
        &human,
        Stats::default(),
    );
    assert_eq!(john.stats(), Stats::default() + 1);
    assert_ne!(john.stats(), Stats::default());

    let mut jill = Character::new(
        String::from("jill"),
        &wizard,
        &acolyte,
        &dwarf,
        Stats::default(),
    );
    let mut other_stats = Stats::default();
    other_stats.constitution += 2;
    assert_eq!(jill.stats(), other_stats);
    jill.race.choose_subrace(0); // hill dwarf
    other_stats.wisdom += 1;
    assert_eq!(jill.stats(), other_stats);
    other_stats.wisdom += 3;
    assert_ne!(jill.stats(), other_stats);
}

#[tokio::test]
async fn char_spells() {
    let provider = provider();
    let wizard_future = provider.get_class("wizard");
    let acolyte_future = provider.get_background("acolyte");
    let elf_future = provider.get_race("elf");

    let spells_futures = vec![
        provider.get_spell("fire bolt"),
        provider.get_spell("shield"),
        provider.get_spell("mage armor"),
    ];

    let (wizard, acolyte, elf) = (
        wizard_future.await.unwrap(),
        acolyte_future.await.unwrap(),
        elf_future.await.unwrap(),
    );

    let stats = Stats::from(&[10, 10, 10, 13, 10, 10]);

    // this is john. john has a base int score of 13, and john is a high elf. His int should be 14.
    let mut john = Character::new(String::from("john"), &wizard, &acolyte, &elf, stats);
    john.race.choose_subrace(0);

    // An int of 14 is a modifier of 2.
    assert_eq!(john.stats().modifiers().stats.intelligence, 2);

    // john should have a spell save dc of 12, and a spell attack modifier of 4.
    let (spell_save, spell_mod) = john
        .spellcasting_scores(0)
        .expect("wizard character should be a spellcaster");
    assert_eq!(spell_save, 12);
    assert_eq!(spell_mod, 4);

    let spells = futures::future::try_join_all(spells_futures)
        .await
        .expect("failed to get spells");

    let (list, spell_amounts) = john
        .prepare_spells(0)
        .expect("wizard should be able to prepare spells");
    assert_eq!(
        spell_amounts.num_cantrips, 3,
        "wizards get 3 cantrips at level 1"
    );
    assert_eq!(
        spell_amounts.num_spells, 3,
        "This wizard can prepare 3 spells"
    );
    list.extend(spells);

    assert!(list.iter().any(|s| s.name == "Fire Bolt"));
    assert!(list.iter().any(|s| s.name == "Shield"));
    assert!(list.iter().any(|s| s.name == "Mage Armor"));

    assert_eq!(john.available_spell_slots, john.spell_slots());
    let spell_slots = john
        .available_spell_slots
        .as_ref()
        .expect("wizard should have spell slots");
    assert_eq!(spell_slots.0, [2, 0, 0, 0, 0, 0, 0, 0, 0]);
    let result = john.cast_prepared(0, "mage armor", None, None);
    assert!(result, "john should be able to cast mage armor");
    let new_spell_slots = john
        .available_spell_slots
        .as_ref()
        .expect("wizard should have spell slots");
    assert_eq!(new_spell_slots.0, [1, 0, 0, 0, 0, 0, 0, 0, 0]);
}

#[tokio::test]
async fn char_multiclassing() {
    let provider = provider();
    let monk_future = provider.get_class("monk");
    let fighter_future = provider.get_class("fighter");
    let acolyte_future = provider.get_background("acolyte");
    let human_future = provider.get_race("human");

    let monk = monk_future.await.unwrap();
    let fighter = fighter_future.await.unwrap();
    let acolyte = acolyte_future.await.unwrap();
    let human = human_future.await.unwrap();

    let stats_bonus_dex = Feature {
        name: String::new(),
        description: vec![],
        effects: vec![FeatureEffect::AddModifier(StatType::Dexterity, 10)],
    };

    let stats_bonus_wis = Feature {
        effects: vec![FeatureEffect::AddModifier(StatType::Wisdom, 10)],
        ..stats_bonus_dex.clone()
    };

    let mut john = Character::new(
        String::from("John"),
        &monk,
        &acolyte,
        &human,
        Stats::default(),
    );

    // should fail, since john doesn't have the necessary stats
    assert_eq!(john.level_up(&fighter), None);
    john.bonus_features.push(stats_bonus_dex.clone());

    // now that john's stats are boosted, john meets the minimum, and john can level up
    assert_eq!(john.level_up(&fighter), Some(1));

    // george is going the other way. George is a fighter multiclassing as a monk.
    let mut george = Character::new(
        String::from("George"),
        &fighter,
        &acolyte,
        &human,
        Stats::default(),
    );

    // should fail, since george doens't have the dex nor wis requirement
    assert_eq!(george.level_up(&monk), None);

    george.bonus_features.push(stats_bonus_dex);

    // still doesn't meet the wisdom requirement
    assert_eq!(george.level_up(&monk), None);

    george.bonus_features.push(stats_bonus_wis);

    // with the requirements finally met, george can multiclass
    assert_eq!(george.level_up(&monk), Some(1));
}

#[tokio::test]
async fn druid_wildshape() {
    let provider = provider();
    let druid_future = provider.get_class("druid");
    let acolyte_future = provider.get_background("acolyte");
    let human_future = provider.get_race("human");

    let druid = druid_future.await.unwrap();
    let acolyte = acolyte_future.await.unwrap();
    let human = human_future.await.unwrap();

    let bingus = Character::new(
        String::from("John"),
        &druid,
        &acolyte,
        &human,
        Stats::default(),
    );
    assert_eq!(bingus.classes[0].class, "Druid");
    assert_eq!(bingus.classes[0].level, 1);
    let etc_fields = &bingus.classes[0].tracked_fields;
    assert_eq!(etc_fields.len(), 1);
    let wildshape = &etc_fields[0];
    assert_eq!(wildshape.1, 2);
    assert_eq!(
        wildshape.0,
        TrackedField {
            name: "Wildshape".to_string(),
            long_rest: true,
            short_rest: true,
            level_up: false,
            class_specific_max: None,
            hard_max: Some(2),
        }
    );
}

#[tokio::test]
async fn barbarian_rage() {
    let provider = provider();
    let barbarian_future = provider.get_class("barbarian");
    let acolyte_future = provider.get_background("acolyte");
    let human_future = provider.get_race("human");

    let barbarian = barbarian_future.await.unwrap();
    let acolyte = acolyte_future.await.unwrap();
    let human = human_future.await.unwrap();

    let mut boko = Character::new(
        String::from("Boko"),
        &barbarian,
        &acolyte,
        &human,
        Stats::default(),
    );
    assert_eq!(boko.classes[0].class, "Barbarian");
    assert_eq!(boko.classes[0].level, 1);

    let etc_fields = &boko.classes[0].tracked_fields;
    assert_eq!(etc_fields.len(), 1);
    let rage = &etc_fields[0];
    assert_eq!(rage.1, 2);
    assert_eq!(
        rage.0,
        TrackedField {
            name: "Rage".to_string(),
            long_rest: true,
            short_rest: false,
            level_up: false,
            class_specific_max: Some("rage count".to_string()),
            hard_max: None,
        }
    );

    boko.level_up_to_level(&barbarian, 11);
    let rage = boko.classes[0].tracked_fields.first().unwrap();
    assert_eq!(rage.1, 4);

    boko.level_up(&barbarian);
    let rage = boko.classes[0].tracked_fields.first().unwrap();
    assert_eq!(rage.1, 5);
}

#[tokio::test]
async fn builder_test() {
    let provider = provider();
    let human_future = provider.get_race("human");
    let druid_future = provider.get_class("druid");
    let acolyte_future = provider.get_background("acolyte");
    let quarterstaff_future = provider.get_item("quarterstaff");

    let human = human_future.await.expect("failed to get human race");
    let druid = druid_future.await.expect("failed to get fighter class");
    let acolyte = acolyte_future
        .await
        .expect("failed to get acolyte background");

    let character = CharacterBuilder::new("TestChar")
        .race(&human)
        .class(&druid)
        .background(&acolyte)
        .stats(Stats::default())
        .build();
    assert!(
        character.is_ok(),
        "Failed to build character: {:?}",
        character.err()
    );

    let quarterstaff = quarterstaff_future
        .await
        .expect("failed to get quarterstaff item");
    let character2_result = CharacterBuilder::new("TestChar2")
        .race(&human)
        .class(&druid)
        .background(&acolyte)
        .stats(Stats::default())
        .choose_items(0, 0)
        .choose_items(1, 1)
        .set_unchosen_category(1, 0, quarterstaff);
    let character2 = match character2_result.clone().build() {
        Ok(c) => c,
        Err(e) => panic!("Failed to build character with item choices: {:?}", e),
    };

    assert_eq!(
        character2.unchosen_items().len(),
        0,
        "There should be no unchosen items left"
    );

    let character_result_race_err = CharacterBuilder::new("TestChar3")
        .class(&druid)
        .background(&acolyte)
        .stats(Stats::default())
        .build();
    assert!(
        character_result_race_err.is_err(),
        "Building character without race should fail"
    );

    let character_result_class_err = CharacterBuilder::new("TestChar4")
        .race(&human)
        .background(&acolyte)
        .stats(Stats::default())
        .build();
    assert!(
        character_result_class_err.is_err(),
        "Building character without class should fail"
    );

    let character_result_background_err = CharacterBuilder::new("TestChar5")
        .race(&human)
        .class(&druid)
        .stats(Stats::default())
        .build();
    assert!(
        character_result_background_err.is_err(),
        "Building character without background should fail"
    );

    let character_result_stats_err = CharacterBuilder::new("TestChar6")
        .race(&human)
        .class(&druid)
        .background(&acolyte)
        .build();
    assert!(
        character_result_stats_err.is_err(),
        "Building character without stats should fail"
    );
}
