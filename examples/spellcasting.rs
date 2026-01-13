use dnd_lib::prelude::*;

// Demonstrating a very simple spellcaster

#[tokio::main]
async fn main() {
    // Initialize the api getter
    let provider = Dnd5eapigetter::new();

    // Fetch our spellcaster class, Wizard
    let wizard = provider.get_class("wizard").await.unwrap();

    // The race will be human, and the background will be acolyte
    let human = provider.get_race("human").await.unwrap();
    let acolyte = provider.get_background("acolyte").await.unwrap();

    // create the spellcaster character
    let mut spellcaster = CharacterBuilder::new("beebo the wise")
        .class(&wizard)
        .race(&human)
        .stats(Stats::default())
        .background(&acolyte)
        .build()
        .unwrap();
    // let's take a look at the spells available to our wizard at level 1
    let spell_slots = spellcaster.spell_slots().unwrap();
    println!("At level 1, {} can cast {} 1st level spells", spellcaster.name, spell_slots.0[0]);
}
