use crate::character::features::{Feature, FeatureEffect};
use crate::character::stats::StatType;
use crate::get::{get_class, get_background, get_race};
use super::stats::Stats;
use super::player_character::Character;

//#[cfg(feature = "network-intensive-tests")]
#[tokio::test]
async fn char_stats() {
    let wizard_future = get_class("wizard");
    let acolyte_future = get_background("acolyte");
    let human_future = get_race("human");
    let dwarf_future = get_race("dwarf");

    let (wizard, acolyte) = (wizard_future.await.unwrap(), acolyte_future.await.unwrap());
    let (human, dwarf) = (human_future.await.unwrap(), dwarf_future.await.unwrap());

    let john = Character::new(String::from("john"), &wizard, &acolyte, &human, Stats::default());
    assert_eq!(john.stats(), Stats::default() + 1); 
    assert_ne!(john.stats(), Stats::default());

    let mut jill = Character::new(String::from("jill"), &wizard, &acolyte, &dwarf, Stats::default());
    let mut other_stats = Stats::default();
    other_stats.constitution += 2;
    assert_eq!(jill.stats(), other_stats);
    // choose
    jill.race.subraces = jill.race.subraces.choose(0).unwrap().clone();
    other_stats.wisdom += 1;
    assert_eq!(jill.stats(), other_stats);
    other_stats.wisdom += 3;
    assert_ne!(jill.stats(), other_stats);
}

//#[cfg(feature = "network-intensive-tests")]
#[tokio::test]
async fn char_spells() {
    let wizard_future = get_class("wizard");
    let acolyte_future = get_background("acolyte");
    let elf_future = get_race("elf");

    let (wizard, acolyte, elf) = (wizard_future.await.unwrap(), acolyte_future.await.unwrap(), elf_future.await.unwrap());

    let stats = Stats::from(&[10, 10, 10, 13, 10, 10]);

    // this is john. john has a base int score of 13, and john is a high elf. His int should be 14.
    let mut john = Character::new(String::from("john"), &wizard, &acolyte, &elf, stats);
    john.race.subraces.choose_in_place(0);

    // An int of 14 is a modifier of 2.
    assert_eq!(john.stats().modifiers().intelligence, 2);

    // john should have a spell save dc of 12, and a spell attack modifier of 4.
    let (spell_save, spell_mod) = john.spellcasting_scores(0).expect("wizard character should be a spellcaster");
    assert_eq!(spell_save, 12);
    assert_eq!(spell_mod, 4);
}

#[tokio::test]
async fn char_multiclassing() {
    let monk_future = get_class("monk");
    let fighter_future = get_class("fighter");
    let acolyte_future = get_background("acolyte");
    let human_future = get_race("human");

    let monk = monk_future.await.unwrap();
    let fighter = fighter_future.await.unwrap();
    let acolyte = acolyte_future.await.unwrap();
    let human = human_future.await.unwrap();
    
    let stats_bonus_dex = Feature {
        name: String::new(),
        description: vec![],
        effects: vec![FeatureEffect::AddModifier(StatType::Dexterity, 10)]
    };

    let stats_bonus_wis = Feature {
        effects: vec![FeatureEffect::AddModifier(StatType::Wisdom, 10)],
        ..stats_bonus_dex.clone()
    };



    let mut john = Character::new(String::from("John"), &monk, &acolyte, &human, Stats::default());


    // should fail, since john doesn't have the necessary stats
    assert_eq!(john.level_up(&fighter), None);
    john.bonus_features.push(stats_bonus_dex.clone());

    dbg!(john.stats());
    // now that john's stats are boosted, john meets the minimum, and john can level up 
    assert_eq!(john.level_up(&fighter), Some(1));


    // george is going the other way. George is a fighter multiclassing as a monk.
    let mut george = Character::new(String::from("George"), &fighter, &acolyte, &human, Stats::default());

    // should fail, since george doens't have the dex nor wis requirement
    assert_eq!(george.level_up(&monk), None);

    george.bonus_features.push(stats_bonus_dex);

    // still doesn't meet the wisdom requirement
    assert_eq!(george.level_up(&monk), None);

    george.bonus_features.push(stats_bonus_wis);

    // with the requirements finally met, george can multiclass
    assert_eq!(george.level_up(&monk), Some(1));
}
