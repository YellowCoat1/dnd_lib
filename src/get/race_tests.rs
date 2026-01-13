use crate::rules2014::stats::{Size, StatType};
use crate::{prelude::*, provider};

#[tokio::test]
async fn get_elf() {
    let provider = provider();
    let elf = provider.get_race("elf").await.expect("failed to get elf!");
    assert_eq!(
        (elf.name(), elf.speed(), *elf.size()),
        ("Elf", 30, Size::Medium)
    );
    assert_eq!(
        elf.ability_bonuses().first().cloned(),
        Some((Some(StatType::Dexterity), 2))
    );
    assert_eq!(
        elf.languages().first().cloned(),
        Some(String::from("Common"))
    );

    let high_elf = elf.subraces().first().expect("Elf should have subraces!");

    assert_eq!(high_elf.name(), "High Elf");
    assert_eq!(
        high_elf.ability_bonuses().first().cloned(),
        Some((Some(StatType::Intelligence), 1))
    );
}

#[tokio::test]
async fn get_dragonborn() {
    let provider = provider();
    let dragonborn = provider
        .get_race("dragonborn")
        .await
        .expect("failed to get dragonborn!");
    assert_eq!((dragonborn.name(), dragonborn.speed()), ("Dragonborn", 30));
    let draconic = dragonborn
        .languages()
        .get(1)
        .expect("Dragonborn should have 2 languages")
        .clone();
    assert_eq!(draconic, "Draconic");
}

async fn get_with_race_context(
    race_name: &str,
    provider: &Dnd5eapigetter,
) -> Result<Race, CharacterDataError> {
    provider
        .get_race(race_name)
        .await
        .map_err(|e| e.prepend(format!("{} ", race_name).as_str()))
}

#[tokio::test]
async fn fetch_all() {
    let provider = provider();
    let races = super::RACE_NAMES
        .iter()
        .map(|n| get_with_race_context(n, provider.as_ref()));

    futures::future::try_join_all(races)
        .await
        .expect("failed to fetch races");
}
