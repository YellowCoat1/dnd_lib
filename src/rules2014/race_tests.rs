use crate::rules2014::{Race, RaceBonus, RaceBuilder, Subrace, SubraceBuilder, choice::PresentedOption, features::{Feature, FeatureEffect}, stats::{Size, StatType}};

#[test]
fn custom_race() {
    use super::stats::StatType::Strength;

    let animated_armor_slumber = Feature {
        name: "Animated Armor Slumber".to_string(),
        description: vec!["As animated armor, rather than sleeping as a creature, you become inanimate".to_string()],
        effects: vec![],
    };

    let (animated_leather, animated_plate) = animated_subraces();
    let mut animated_armor: Race = RaceBuilder::new("animated armor")
        .size(Size::Large)
        .speed(25)
        .add_ability_bonus(RaceBonus::Specific(Strength, 2))
        .add_ability_bonus(RaceBonus::Wildcard(1))
        .add_trait(PresentedOption::Base(animated_armor_slumber))
        .add_language("Elvish".to_string())
        .add_wildcard_language()
        .add_subrace(animated_leather)
        .build();

    assert_eq!(animated_armor.name(), "Animated Armor");
    assert_eq!(animated_armor.speed(), 25);
    assert_eq!(*animated_armor.size(), Size::Large);
    assert_eq!(*animated_armor.ability_bonuses(), vec![RaceBonus::Specific(Strength, 2), RaceBonus::Wildcard(1)]);
    assert_eq!(animated_armor.traits().first().expect("Armor should have traits").as_base().unwrap().name, "Animated Armor Slumber");
    assert_eq!(*animated_armor.languages(), vec!["Elvish"]);
    assert_eq!(animated_armor.wildcard_languages(), 1);

    // adding a new subrace to it
    animated_armor.add_subrace(animated_plate);
    assert_eq!(animated_armor.subraces().len(), 2);
    
    let animated_armor2 = animated_armor.clone();
    assert_eq!(animated_armor, animated_armor2);
}

fn animated_subraces() -> (Subrace, Subrace) {
    let animated_leather = SubraceBuilder::new("Animated Leather")
        .description("Animated Leather Armor".to_string())
        .build();
    let animated_plate_toughness = Feature {
        name: "Animated Plate Toughness".to_string(),
        description: vec!["As animated plate of a metal armor, your AC is naturally increased.".to_string()],
        effects: vec![FeatureEffect::ACBonus(2)],
    };
    let animated_plate = SubraceBuilder::new("Animated Plate")
        .add_ability_bonus(Some(StatType::Constitution), 1)
        .add_trait(PresentedOption::Base(animated_plate_toughness))
        .build();
    assert_eq!(animated_leather, animated_leather);
    assert_ne!(animated_leather, animated_plate);
    (animated_leather, animated_plate)
}
