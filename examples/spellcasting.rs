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

    // let's take a look at the spells available to our druid at level 3
    let spell_slots = spellcaster.spell_slots().unwrap();
    println!("At level 1, {} can cast {} 1st level spells", spellcaster.name, spell_slots.0[0]);
    println!("At level 2, {} can cast {} 2nd level spells", spellcaster.name, spell_slots.0[1]);

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

    // how many spells can the druid prepare?
    let spells_to_prepare = spellcaster.num_spells(0).unwrap(); 
    println!("At level 3, {} can prepare {} spells and {} cantrips ", spellcaster.name, spells_to_prepare.num_spells, spells_to_prepare.num_cantrips);

    // we want to get the spells we want to prepare

}
