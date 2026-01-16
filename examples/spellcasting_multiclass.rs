use dnd_lib::prelude::*;

// A level 5 spellcasting character with 3 levels in cleric, and 2 levels in sorcerer.

#[tokio::main]
async fn main() {
    // Initialize the api getter
    let provider = Dnd5eapigetter::new();

    println!("Fetching data...");
    let elf_future = provider.get_race("elf");
    let cleric_future = provider.get_class("cleric");
    let sorcerer_future = provider.get_class("sorcerer");
    let acolyte_future = provider.get_background("acolyte");

    let elf = elf_future.await.unwrap();
    let cleric = cleric_future.await.unwrap();
    let sorcerer = sorcerer_future.await.unwrap();
    let acolyte = acolyte_future.await.unwrap();
    println!("Data fetched.");

    let stats = Stats {
        strength: 8,
        dexterity: 10,
        constitution: 13,
        intelligence: 12,
        wisdom: 14,
        charisma: 15,
    };

    let mut spellcaster = CharacterBuilder::new("Spellcaster")
        .class(&cleric)
        .race(&elf)
        .background(&acolyte)
        .stats(stats)
        .build()
        .expect("Failed to build character");

    // choose high elf subrace
    spellcaster.race.choose_subrace(0);
    
    // level up cleric to level 3, and then multiclass to sorcerer level 2
    spellcaster.level_up_to_level(&cleric, 3);
    spellcaster.level_up_multiple(&sorcerer, 2).expect("Failed to multiclass");
}
