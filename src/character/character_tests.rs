#![cfg(feature = "network-intensive-tests")]
use super::player_character::Character;
use super::stats::Stats;
use crate::character::class::TrackedField;
use crate::character::features::{Feature, FeatureEffect};
use crate::character::stats::StatType;
use crate::getter::DataProvider;

use crate::provider;

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

    dbg!(john.stats());
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
