use crate::character::{
    class::{Class, ItemCategory}, features::PresentedOption, spells::SpellSlots, stats::SkillType
};

use super::{
  class::get_class,
  item::get_item,
  feature::get_feature,
};
   
#[tokio::test]
async fn wizard_retrieval() {
    let wizard = get_class("wizard")
        .await
        .expect("failed to get wizard class from api");

    let items_result = wizard_items(&wizard);
    let features_result = wizard_features(&wizard);

    wizard_skill_proficiencies(&wizard);
    wizard_spells(&wizard);
    wizard_class_specific(&wizard);
    wizard_subclass(&wizard);

    items_result.await;
    features_result.await;
}


fn wizard_skill_proficiencies(class: &Class) {
   use PresentedOption::{Base, Choice};
   use SkillType::{Arcana, History, Insight, Investigation, Medicine, Religion};

   let proficiencies: Vec<PresentedOption<SkillType>> = vec![
       Base(Arcana),
       Base(History),
       Base(Insight),
       Base(Investigation),
       Base(Medicine),
       Base(Religion),
   ];

   assert_eq!(class.skill_proficiency_choices.1, Choice(proficiencies), "Couldn't retrieve proper wizard proficiencies");
}

async fn wizard_items(class: &Class) {
    // gets the spellbook item
    // shouldn't cause an api call due to the memoization
    let spellbook_item = get_item("spellbook")
        .await.unwrap();
    let spellbook_choice_entry = PresentedOption::Base(vec![(ItemCategory::Item(spellbook_item), 1)]);
    assert!(class.beginning_items.contains(&spellbook_choice_entry), "class invalid items");
}

async fn wizard_features(class: &Class) {
    let wizard_spellcasting_feature = get_feature("spellcasting-wizard").await.unwrap();
    let wizard_feature = class.features
    .get(0)
    .expect("Wizard has no features")
    .get(0)
    .expect("Wizard has no features on level 1");
    assert_eq!(*wizard_feature, PresentedOption::Base(wizard_spellcasting_feature), "Wizard has invalid feature set");
}

fn wizard_spells(class: &Class) {
    let level_one_spell_slots = class.spellcasting.as_ref().expect("wizard should have spells").spell_slots_per_level
        .get(5-1)
        .expect("Wizard doesn't have spell slots at level 5");

    let expected_spell_slots = SpellSlots([4, 3, 2, 0, 0, 0, 0, 0, 0]);

    assert_eq!(*level_one_spell_slots, expected_spell_slots, "Wizard has wrong spell slots");
}

fn wizard_class_specific(class: &Class) {
    let arcane_recovery = class.class_specific_leveled.get("arcane recovery levels").expect("wizard should have class specific fields!");
    let arcane_recovery_nums: Vec<usize> = arcane_recovery.iter().map(|v| v.parse().unwrap()).collect();

    assert_eq!(arcane_recovery_nums, vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10])
}

fn wizard_subclass(class: &Class) {
    let evocation = class.subclasses.get(0).expect("wizard should have subclasses!");
    assert_eq!(evocation.name, "Evocation");
}
