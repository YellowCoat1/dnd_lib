use dnd_lib::{prelude::*, rules2014::features::PresentedOption};

// An example of a simple level 1 character, with all essential choices.

#[tokio::main]
async fn main() {
    // creating an api getter, which will fetch data from the 5e api
    let provider = Dnd5eapigetter::new();

    // fetching the race "elf"
    let elf = provider
        .get_race("elf")
        // wait for it to complete getting from the api
        .await
        // panic if there was an error
        .expect("Failed to get elf");
    println!("got race");

    // same for the class
    let druid = provider
        .get_class("druid")
        .await
        .expect("Failed to get druid");
    println!("got class");

    // and the background
    let acolyte = provider
        .get_background("acolyte")
        .await
        .expect("Failed to get acolyte");

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
    let mut george = CharacterBuilder::new("George") // first, creating the builder with the name of the character
        .race(&elf) // making george an elf
        .class(&druid) // making george a druid
        .background(&acolyte) // giving george the acolyte background
        .stats(stats) // setting george's stats
        .build() // finally, build the character
        .unwrap(); // panic if there was an error in that process

    println!("Created character: {:#?}", george.name);

    // we want to select the items that george can select from his class and background.

    // first, we get the availible items.
    let unchosen_items = george.unchosen_items();

    println!("Items to choose from:");
    // this prints out all the unchosen items
    for item in unchosen_items.iter() {
        // for every item choice
        print!(" - ");
        match item {
            PresentedOption::Base(b) => {
                // if it's chosen,
                for (i, item) in b.iter().enumerate() {
                    // list all the items
                    print!("{}", item.0);
                    if i != b.len() - 1 {
                        // add a comma inbetween items
                        print!(", ");
                    }
                }
                println!();
            }
            PresentedOption::Choice(choices_list) => {
                // if it's unchosen, (which they all should be, it is "unchosen_items" for a reason)
                for (index, option) in choices_list.iter().enumerate() {
                    // for every option
                    for (index2, item) in option.iter().enumerate() {
                        // list the items
                        print!("{}", item.0);
                        if index2 != option.len() - 1 {
                            // add a comma imbetween items
                            print!(", ");
                        }
                    }
                    if index != choices_list.len() - 1 {
                        // add an "or" inbetween options
                        print!(" or ");
                    }
                }
                println!();
            }
        }
    }

    // choose the shield
    // this is from the first choice, and within that choice the shield is the first option
    george.choose_items(0, 0);

    // choose the quarterstaff for the second choice
    // this is the second choice, and within that choice "Any simple melee weapon" is the second option
    george.choose_items(1, 1);

    // get the quarterstaff item
    let quarterstaff = provider
        .get_item("quarterstaff")
        .await
        .expect("Failed to get quarterstaff");

    // set the quarterstaff as the unchosen item for the second choice
    // notice how the choice index is now 0.
    // Because we set the choice already, that choice is now the one and only choice, so its index is 0.
    let result = george.set_unchosen_category(1, 0, quarterstaff);
    assert!(result, "Failed to give the druid a quarterstaff");

    // finally, submit these choices and give the character these items.
    george.add_chosen_items();

    // selecting the subrace for george
    let subraces = george.race.subraces();
    println!("Subraces to choose from:");
    for subrace in subraces.choices().unwrap().iter() {
        println!(" - {}", subrace.name());
    }

    // choosing the high elf subrace (the first, and only one)
    george.race.choose_subrace(0);
}
