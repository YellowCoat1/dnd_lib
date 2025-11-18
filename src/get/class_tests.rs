use crate::character::{
    class::{Class, ItemCategory},
    features::PresentedOption,
    stats::SkillType,
};

use crate::provider;

use crate::getter::DataProvider;

use super::{feature::get_feature, item::get_item};

#[tokio::test]
async fn wizard_retrieval() {
    let provider = provider();
    let wizard = provider
        .get_class("wizard")
        .await
        .expect("failed to get wizard class from api");

    let items_result = wizard_items(&wizard);
    let features_result = wizard_features(&wizard);

    wizard_skill_proficiencies(&wizard);
    wizard_class_specific(&wizard);
    wizard_subclass(&wizard);
    wizard_multiclassing(&wizard);

    items_result.await;
    features_result.await;
}

fn wizard_multiclassing(class: &Class) {
    use crate::character::stats::{EquipmentProficiencies, StatType};

    assert_eq!(
        class.multiclassing_proficiency_gain,
        EquipmentProficiencies::default()
    );

    let v = StatType::from_shorthand("int");
    let multiclassing_prerequisites = vec![(v.as_ref().unwrap(), &13)];

    assert_eq!(
        class.multiclassing_prerequisites.iter().collect::<Vec<_>>(),
        multiclassing_prerequisites,
        "wizard multiclassing prerequisites are incorrect"
    );
}

fn wizard_skill_proficiencies(class: &Class) {
    use PresentedOption::Choice;
    use SkillType::{Arcana, History, Insight, Investigation, Medicine, Religion};

    let proficiencies: Vec<SkillType> =
        vec![Arcana, History, Insight, Investigation, Medicine, Religion];

    assert_eq!(
        class.skill_proficiency_choices.1,
        Choice(proficiencies),
        "Couldn't retrieve proper wizard proficiencies"
    );
}

async fn wizard_items(class: &Class) {
    let spellbook_item = get_item("spellbook").await.unwrap();

    let spellbook_choice_entry =
        PresentedOption::Base(vec![(ItemCategory::Item(spellbook_item), 1)]);
    assert_eq!(
        class.beginning_items.first().cloned(),
        Some(spellbook_choice_entry)
    );
    let first_choice = class
        .beginning_items
        .get(1)
        .expect("Wizard should have more than one item")
        .choices()
        .expect("Wizard's second item list should be a choice");

    assert_eq!(
        first_choice.len(),
        2,
        "Wizard's first item choice should be between two items"
    );
    let quarterstaff = get_item("quarterstaff")
        .await
        .expect("Couldn't get quarterstaff");
    let dagger = get_item("dagger").await.expect("Couldn't get dagger");
    assert_eq!(first_choice[0], vec![(ItemCategory::Item(quarterstaff), 1)]);
    assert_eq!(first_choice[1], vec![(ItemCategory::Item(dagger), 1)]);
}

async fn wizard_features(class: &Class) {
    let wizard_spellcasting_feature = get_feature("spellcasting-wizard").await.unwrap();
    let wizard_feature = class
        .features
        .first()
        .expect("Wizard has no features")
        .first()
        .expect("Wizard has no features on level 1");
    assert_eq!(
        *wizard_feature,
        PresentedOption::Base(wizard_spellcasting_feature),
        "Wizard has invalid feature set"
    );
}

fn wizard_class_specific(class: &Class) {
    let arcane_recovery = class
        .class_specific_leveled
        .get("arcane recovery levels")
        .expect("wizard should have class specific fields!");
    let arcane_recovery_nums: Vec<usize> =
        arcane_recovery.iter().map(|v| v.parse().unwrap()).collect();

    assert_eq!(
        arcane_recovery_nums,
        vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10]
    )
}

fn wizard_subclass(class: &Class) {
    let evocation = class
        .subclasses
        .first()
        .expect("wizard should have subclasses!");
    assert_eq!(evocation.name, "Evocation");
}

#[tokio::test]
async fn fighter_items() {
    let provider = provider();
    let fighter = provider
        .get_class("fighter")
        .await
        .expect("failed to get warlock class from api");

    assert_eq!(
        fighter.beginning_items.len(),
        4,
        "Fighter should have 4 item choices"
    );

    let first_choice = fighter.beginning_items[0]
        .choices()
        .expect("Fighter should have beginning choices");
    assert_eq!(
        first_choice.len(),
        2,
        "Fighter's first option should have 2 choice"
    );

    assert_eq!(
        first_choice[0].len(),
        1,
        "The first choice of fighter's first option should have 1 item"
    );
    match &first_choice[0][0].0 {
        ItemCategory::Item(i) => assert_eq!(i.name, "Chain Mail"),
        _ => panic!("Item should be an item, not a category"),
    }
    assert_eq!(
        first_choice[0][0].1, 1,
        "Fighter's first choice in the first option should have a count of 1"
    );

    assert_eq!(
        first_choice[1].len(),
        3,
        "The second choice of fighter's first option should have 3 item"
    );
    match &first_choice[1][0] {
        (ItemCategory::Item(i), 1) => assert_eq!(i.name, "Leather Armor"),
        (ItemCategory::Item(_), _) => panic!("Fighter should have only 1 leather armor"),
        _ => panic!("Leather armor should be an item, not a category"),
    }
    match &first_choice[1][1] {
        (ItemCategory::Item(i), 1) => assert_eq!(i.name, "Longbow"),
        (ItemCategory::Item(_), _) => panic!("Fighter should have only 1 longbow"),
        _ => panic!("Longbow should be an item, not a category"),
    }
    match &first_choice[1][2] {
        (ItemCategory::Item(i), 20) => assert_eq!(i.name, "Arrow"),
        (ItemCategory::Item(_), _) => panic!("Fighter should have 30 arrows"),
        _ => panic!("Arrows should be items, not a category"),
    }

    let second_choice = fighter.beginning_items[1]
        .choices()
        .expect("Fighter's second item field should be a choice");
    assert_eq!(
        second_choice.len(),
        2,
        "Fighter's second item field should have 2 choices"
    );

    assert_eq!(second_choice[1].len(), 1, "the latter part of fighter's second item field should be only one item (2 martial weapons, 1 unique item type)");
    let two_martial_weapons = &second_choice[1][0];
    assert_eq!(
        two_martial_weapons.0,
        ItemCategory::Weapon(crate::character::items::WeaponType::Martial),
        "Fighter should be able to choose martial weapons"
    );
    assert_eq!(
        two_martial_weapons.1, 2,
        "Fighter should be able to choose 2 martial weapons"
    );

    // proficiencies. ew.
    let (n, _ops) = fighter.skill_proficiency_choices;
    assert_eq!(n, 2);
}
