use dnd_lib::{character::features::PresentedOption, prelude::*};

#[tokio::main]
async fn main() {
    // creating an api getter.
    let provider = Dnd5eapigetter::new();
    // fetching the race "elf"
    let elf = provider
        .get_race("elf")
        // wait for it to complete getting from the api
        .await
        // panic if there was an error
        .unwrap();
    println!("got race");
    // same for the class
    let druid = provider.get_class("druid").await.unwrap();
    println!("got class");
    // and the background
    let acolyte = provider.get_background("acolyte").await.unwrap();
    println!("got background");

    // some basic stats for our character
    let stats = Stats {
        strength: 10,
        dexterity: 14,
        constitution: 12,
        intelligence: 13,
        wisdom: 15,
        charisma: 11,
    };

    // now we can create our character
    let george = CharacterBuilder::new("George") // first, creating the builder with the name of the character
        .race(&elf) // making george an elf
        .class(&druid) // making george a druid
        .background(&acolyte) // giving george the acolyte background
        .stats(stats) // setting george's stats
        .build() // finally, build the character
        .unwrap(); // panic if there was an error in that process

    println!("Created character: {:#?}", george.name);

    // we want to select the items that george can select from his class and background.

    // first, we get the availible items.
    let _unchosen_items = george.unchosen_items();

    //
    //    println!("Items to choose from:");
    //    for item in unchosen_items.iter() {
    //        match item {
    //            PresentedOption::Base(b) => {
    //                for item in b.iter() {
    //                    print!("a {}, ", item.0.name);
    //                }
    //            }
    //            PresentedOption::Choice(c) => {
    //                for option in c.iter()
    //            }
    //        }
    //    }
    //
}
