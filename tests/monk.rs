#![cfg(feature = "network-intensive-tests")]
use dnd_lib::character::stats::{SkillModifiers, SkillType};
use dnd_lib::prelude::*;

#[tokio::test]
async fn level_3_elf_monk() {
    let provider = Dnd5eapigetter::new();
    let elf_future = provider.get_race("elf");
    let monk_future = provider.get_class("monk");
    let acolyte_future = provider.get_background("acolyte");

    let elf = elf_future.await.expect("couldn't get human");
    let monk = monk_future.await.expect("couldnt't get monk");
    let acolyte = acolyte_future.await.expect("couldn't get acolyte");

    // Chosen using standard array
    let stats = Stats {
        strength: 10,
        dexterity: 14,
        constitution: 13,
        intelligence: 12,
        wisdom: 15,
        charisma: 8,
    };

    // georg is level 1
    let mut georg = Character::new("gerog".to_string(), &monk, &acolyte, &elf, stats);

    // add class items
    georg.choose_items(0, 0);
    georg.choose_items(1, 0);
    georg.add_chosen_items();

    // equip the shortsword
    georg
        .items
        .get_mut(3)
        .expect("Items were not added correctly")
        .equip();

    assert_eq!(georg.items.len(), 5);

    assert_eq!(georg.items[0].item.name, "Clothes, common");
    assert_eq!(georg.items[3].item.name, "Shortsword");
    assert_eq!(georg.items[4].item.name, "Dungeoneer's Pack");

    // Choosing the skills we want
    georg.class_skill_proficiencies[0].choose_in_place(0);
    georg.class_skill_proficiencies[1].choose_in_place(1);

    // double checking they have exactly the skills we select, and the skill proficiencies granted
    // by the background
    let skills = georg.skills();
    let s_with_prof = skills
        .skills_with_proficiency()
        .iter()
        .map(|v| v.0)
        .collect::<Vec<_>>();
    assert!(
        s_with_prof.contains(&SkillType::Acrobatics),
        "Character did not have a proficiency in Acrobatics"
    );
    assert!(
        s_with_prof.contains(&SkillType::Athletics),
        "Character did not have a proficiency in Athletics"
    );
    assert!(
        s_with_prof.contains(&SkillType::Insight),
        "Character did not have a proficiency in Insight"
    );
    assert!(
        s_with_prof.contains(&SkillType::Religion),
        "Character did not ahve a proficiency in Religion"
    );

    // choosing the languages
    let lang_options = georg.background.language_options();
    assert_eq!(
        lang_options.len(),
        2,
        "Acolyte should have 2 language options"
    );
    georg.background.choose_language_option(0, "Dwarvish");
    georg.background.choose_language_option(1, "Draconic");

    let langs = georg.total_languages();
    assert!(langs.contains("Common"));
    assert!(langs.contains("Elvish"));
    assert!(langs.contains("Dwarvish"));
    assert!(langs.contains("Draconic"));

    // choosing the subrace
    // there's only one option, (high elf) so we just choose the one available
    georg.race.choose_subrace(0);

    // making sure the monk has zero ki points at level 1
    assert_eq!(
        georg.classes[0].tracked_fields.len(),
        1,
        "monk should have 1 etc field at level 1"
    );
    assert_eq!(
        georg.classes[0].tracked_fields[0].1, 0,
        "monk should have 0 ki points at level 1"
    );

    // level georg to level 3
    georg.level_up_to_level(&monk, 3);
    assert_eq!(georg.level(), 3);

    // monk should have 3 ki points at level 3
    assert_eq!(
        georg.classes[0].tracked_fields.len(),
        1,
        "monk should have 1 etc field at level 3"
    );
    assert_eq!(
        georg.classes[0].tracked_fields[0].1, 3,
        "monk should have 3 ki points at level 3"
    );

    // double checking stats
    let final_stats = Stats {
        strength: 10,
        dexterity: 16,
        constitution: 13,
        intelligence: 13,
        wisdom: 15,
        charisma: 8,
    };
    assert_eq!(georg.stats(), final_stats);

    // double checking skills
    let skills = georg.skill_modifiers();
    let correct_skills = SkillModifiers {
        acrobatics: 5,
        animal_handling: 2,
        arcana: 1,
        athletics: 2,
        deception: -1,
        history: 1,
        insight: 4,
        intimidation: -1,
        investigation: 1,
        medicine: 2,
        nature: 1,
        perception: 2,
        performance: -1,
        persuasion: -1,
        religion: 3,
        sleight_of_hand: 3,
        stealth: 3,
        survival: 2,
    };

    assert_eq!(skills, correct_skills);

    let ac = georg.ac();
    assert_eq!(ac, 15);
}
