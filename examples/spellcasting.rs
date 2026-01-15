use dnd_lib::prelude::*;

// Demonstrating a very simple spellcaster

#[tokio::main]
async fn main() {
    // Initialize the api getter
    let provider = Dnd5eapigetter::new();


    // The race will be human, and the background will be acolyte
    let human = provider.get_race("human").await.unwrap();
    println!("Got human race");
    let acolyte = provider.get_background("acolyte").await.unwrap();
    println!("Got acolyte background");
    // Fetch our spellcaster class, Druid
    let druid = provider.get_class("druid").await.unwrap();
    println!("Got druid class");

    // giving our spellcaster a decent wisdom score, and 10 for everything else
    let stats = Stats {
        wisdom: 16,
        ..Default::default()
    };
    // create the spellcaster character
    let mut spellcaster = CharacterBuilder::new("beebo the wise")
        .class(&druid)
        .race(&human)
        .stats(stats)
        .background(&acolyte)
        .build()
        .expect("Failed to build spellcaster character");
    println!("Created spellcaster: {:?}", spellcaster.name);

    // leveling them up to level 3
    spellcaster.level_up_to_level(&druid, 3);
    assert_eq!(spellcaster.level(), 3);
    // getting the spell save dc and the spell attack bonus
    let (spell_save_dc, spell_attack_bonus) = spellcaster.spellcasting_scores(0).unwrap();
    println!("{} has a spell save DC of {} and a spell attack bonus of {}", spellcaster.name, spell_save_dc, spell_attack_bonus);
    
    // What's the druid spell list?
    let druid_spellcasting = druid.spellcasting().unwrap();
    let cantrips = &druid_spellcasting.spell_list[0];
    let level_1_spells = &druid_spellcasting.spell_list[1];

    // print out the first 5 cantrips
    println!("First 5 Druid Cantrips:");
    for cantrip in cantrips[..5].iter() {
        println!("- {}", cantrip);
    }

    println!("First 5 Druid Level 1 Spells:");
    for spell in level_1_spells[..5].iter() {
        println!("- {}", spell);
    }

    println!("First 5 Druid Level 2 Spells:");
    let level_2_spells = &druid_spellcasting.spell_list[2];
    for spell in level_2_spells[..5].iter() {
        println!("- {}", spell);
    }
   // we want to get the spells we want to prepare
    println!("Getting spells...");
    let spells = vec![
        // cantrips
        provider.get_spell("guidance"),
        provider.get_spell("produce flame"),
        // level 1 spells
        provider.get_spell("charm person"),
        provider.get_spell("cure wounds"),
        provider.get_spell("detect magic"),
        // level 2 spells
        provider.get_spell("barkskin"),
        provider.get_spell("darkvision"),
        provider.get_spell("enhance ability"),
    ];
    let spell_results = futures::future::try_join_all(spells)
        .await
        .expect("Failed to fetch spells");
    println!("Fetched spells.");

    // getting the info for preparing spells
    let (spell_list, max_spells) = spellcaster.prepare_spells(0).unwrap();
    println!("They can prepare {} spells and {} cantrips", max_spells.num_spells, max_spells.num_cantrips);

    // add the fetched spells to the spell list
    spell_list.extend(spell_results.clone());

    // list the prepared spells
    println!("Prepared Spells:");
    for (spell, _) in spellcaster.spells() {
        println!("- {}", spell.name);
    }

    // let's take a look at the spells available to our druid at level 3
    let spell_slots = spellcaster.spell_slots().unwrap();
    println!("{} has {} 1st level slots and {} 2nd level slots.", spellcaster.name, spell_slots.0[0], spell_slots.0[1]);

    // casting a spell
    spellcaster.cast_prepared(0, "detect magic", None, None);
    spellcaster.cast_prepared(0, "cure wounds", Some(2), None);
    println!("{} casts detect magic, and cure wounds upcasted to level 2.", spellcaster.name);

    let new_spell_slots = spellcaster.available_spell_slots.as_ref().unwrap();
    println!("After casting, {} has {} 1st level slots and {} 2nd level slot left.", spellcaster.name, new_spell_slots.0[0], new_spell_slots.0[1]);
}
