use crate::character::{
    class::{Class, ItemCategory}, 
    features::PresentedOption, 
    stats::SkillType
};

use crate::provider;

use crate::getter::DataProvider;

use super::{
  item::get_item,
  feature::get_feature,
};
   
#[tokio::test]
async fn wizard_retrieval() {
    let provider = provider();
    let wizard = provider.get_class("wizard")
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

    assert_eq!(class.multiclassing_proficiency_gain, EquipmentProficiencies::default());
    
    let v = StatType::from_shorthand("int");
    let multiclassing_prerequisites = vec![
        (v.as_ref().unwrap(), &13),
    ];

    assert_eq!(class.multiclassing_prerequisites.iter().collect::<Vec<_>>(), multiclassing_prerequisites, "wizard multiclassing prerequisites are incorrect");
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
    .first()
    .expect("Wizard has no features")
    .first()
    .expect("Wizard has no features on level 1");
    assert_eq!(*wizard_feature, PresentedOption::Base(wizard_spellcasting_feature), "Wizard has invalid feature set");
}

fn wizard_class_specific(class: &Class) {
    let arcane_recovery = class.class_specific_leveled.get("arcane recovery levels").expect("wizard should have class specific fields!");
    let arcane_recovery_nums: Vec<usize> = arcane_recovery.iter().map(|v| v.parse().unwrap()).collect();

    assert_eq!(arcane_recovery_nums, vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10])
}

fn wizard_subclass(class: &Class) {
    let evocation = class.subclasses.first().expect("wizard should have subclasses!");
    assert_eq!(evocation.name, "Evocation");
}


#[tokio::test]
async fn warlock_retrieval() {
    let provider = provider();
    let warlock = provider.get_class("warlock")
        .await
        .expect("failed to get warlock class from api");

    let _ = warlock.spellcasting.expect("Warlock should have spellcasting info");
}
