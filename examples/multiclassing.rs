use dnd_lib::prelude::*;

// Demonstrating how to multiclass a character

#[tokio::main]
async fn main() {
    let provider = Dnd5eapigetter::new();

    // fetching the 2 classes we want: monk and rogue
    let monk = provider.get_class("monk").await.unwrap();
    let rogue = provider.get_class("rogue").await.unwrap();

    // fetching the race our character will be: human
    let human = provider.get_race("human").await.unwrap();

    // fetching the background our character will have: acolyte
    let acolyte = provider.get_background("acolyte").await.unwrap();

    let stats = Stats::default() + 5; // default stats plus 5 added to all stats
                                      // elevated so it's possible to multiclass

    // creating the character using the builder
    let mut character = CharacterBuilder::new("Rez")
        .class(&monk) // they will initially be a monk
        .background(&acolyte) // with the acolyte background
        .race(&human)
        .stats(stats)
        .build()
        .unwrap();

    assert_eq!(character.level(), 1); // they're a level 1 monk
    assert_eq!(character.classes.len(), 1); // they have only one class

    character
        .level_up(&monk)
        .expect("Failed to level up as monk"); // leveling up as monk to
                                               // level 2
    assert_eq!(character.level(), 2); // now they're level 2
    assert_eq!(character.classes.len(), 1); // they still only have one class: monk

    character
        .level_up(&rogue)
        .expect("Failed to level up as rogue"); // now they multiclass to rogue
    assert_eq!(character.level(), 3); // now they're level 3
    assert_eq!(character.classes.len(), 2); // they now have two classes: monk and rogue
}
