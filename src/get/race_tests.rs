use super::race::get_race;
use crate::character::{features::PresentedOption, stats::StatType};

#[tokio::test]
async fn get_elf() {
    let elf = get_race("elf").await.expect("failed to get elf!");
    assert_eq!((elf.name, elf.speed, elf.size), ("Elf".to_string(), 30, "Medium".to_string()));
    assert_eq!(elf.ability_bonuses.get(0).cloned(), Some((StatType::Dexterity, 2)));
    assert_eq!(elf.languages.get(0).cloned(), Some(String::from("Common")));

    let subraces = match elf.subraces {
        PresentedOption::Base(_) => panic!("Elf should have a subrace!"),
        PresentedOption::Choice(c) => c,
    };

    let high_elf = match subraces.get(0).expect("Elf should have subraces!") {
        PresentedOption::Base(b) => b,
        PresentedOption::Choice(_) => panic!("Elf should not have recursive subraces!")
    };

    assert_eq!(high_elf.name.as_str(), "High Elf");
    assert_eq!(high_elf.ability_bonuses.get(0).cloned(), Some((StatType::Intelligence, 1)));
}


#[tokio::test]
async fn get_dragonborn() {
    let dragonborn = get_race("dragonborn").await.expect("failed to get dragonborn!");
    assert_eq!((dragonborn.name, dragonborn.speed), ("Dragonborn".to_string(), 30));
    let draconic = dragonborn.languages.get(1).expect("Dragonborn should have 2 languages").clone();
    assert_eq!(draconic, "Draconic");
}
