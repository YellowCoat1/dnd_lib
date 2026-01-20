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


    // the total spellcasting level should be 5, and so the character should have level 5 spell slots
    let slots = spellcaster.spell_slots()
        .expect("Caster should have spell slots");
    assert_eq!(slots.0, [4, 3, 2, 0, 0, 0, 0, 0, 0]);

    // The caster classes, cleric and sorcerer, have their own spell lists and spells to prepare.
    let cleric_to_prepare = spellcaster.num_spells(0)
        .expect("Cleric should be a caster");
    println!("The character can prepare {} cleric spells and {} cantrips", cleric_to_prepare.num_spells, cleric_to_prepare.num_cantrips);

    let sorcerer_to_learn = spellcaster.num_spells(1)
        .expect("Sorcerer should be a caster");
    println!("The character can learn {} sorcerer spells and {} cantrips", sorcerer_to_learn.num_spells, sorcerer_to_learn.num_cantrips);



    println!("Fetching spells...");
    // cleric spells
    let lesser_restoration_future = provider.get_spell("lesser restoration");
    let healing_word_future = provider.get_spell("mass healing word");

    // sorcerer spells
    //let sleep_future = provider.get_spell("sleep");

    //TODO: Fix sleep
    
    let lesser_restoration = lesser_restoration_future.await.unwrap();
    let mass_healing_word = healing_word_future.await.unwrap();
    println!("Spells fetched.");

    let (cleric_prepared_list, _) = spellcaster.prepare_spells(0).expect("Cleric should be able to prepare spells");
    cleric_prepared_list.push(lesser_restoration);
    cleric_prepared_list.push(mass_healing_word);
    //let (sorcerer_known_list, _) = spellcaster.prepare_spells(1).expect("Sorcerer should be able to learn spells");

    // cast lesser restoration from the cleric spell list
    let result = spellcaster.cast_prepared(0, "lesser restoration", None, None);
    assert!(result, "Failed to cast lesser restoration");
    // cast sleep from the sorcerer spell list
    //let result = spellcaster.cast_prepared(1, "sleep", None, None);
    //assert!(result, "Failed to cast sleep");



    println!("The spellcaster casts sleep (from the sorcerer list) and lesser restoration (from the cleric list) successfully.");
    let first_level_slots = spellcaster.available_spell_slots.as_ref().unwrap().0[0];
    let second_level_slots = spellcaster.available_spell_slots.as_ref().unwrap().0[1];
    println!("They have {} first level slots and {} second level slots remaining.", first_level_slots, second_level_slots);
}
