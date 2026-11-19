use super::{
    background::BackgroundBuildError,
    choice::PresentedOption,
    features::Feature,
    items::{Item, ItemCount},
    stats::SkillType,
};

use super::background::{BackgroundBuilder, LanguageOption};

#[test]
fn test_background_builder_success() {
    use super::{choice::PresentedOption, stats::SkillType};

    let lang_option_1 =
        LanguageOption::new_named_choice(vec!["Common".to_string(), "Elvish".to_string()]);
    let lang_option_2 = LanguageOption::Fixed(String::new());

    let item = Item {
        name: "Test Item".to_string(),
        description: None,
        item_type: super::items::ItemType::Misc,
        features: vec![],
    };

    let item_count = ItemCount {
        item: item.clone(),
        count: 1,
    };

    let feature = Feature {
        name: "Test Feature".to_string(),
        description: vec![],
        effects: vec![],
    };

    let background_result = BackgroundBuilder::new("Test Background")
        .add_proficiency(PresentedOption::Base(SkillType::Deception))
        .add_language_option(lang_option_1.clone())
        .add_language_option(lang_option_2.clone())
        .add_personality_trait("I don't like some things".to_string())
        .add_personality_trait("I enjoy adventures".to_string())
        .add_ideal("Freedom".to_string())
        .add_bond("My family".to_string())
        .add_flaw("I have a quick temper".to_string())
        .add_equipment(item, 1)
        .add_equipment_count(item_count)
        .add_features(vec![feature.clone()])
        .build();

    let bg = match background_result {
        Ok(bg) => bg,
        Err(e) => panic!("Failed to build background: {:?}", e),
    };

    assert_eq!(bg.name(), "Test Background");
    assert_eq!(bg.proficiencies().len(), 1);
    assert_eq!(bg.personality_traits().len(), 2);
    let language_options = bg.language_options();
    assert_eq!(language_options.len(), 2);
    assert_eq!(language_options[0], lang_option_1);
    assert_eq!(language_options[1], lang_option_2);
    assert_eq!(*bg.features(), vec![feature]);
    assert_eq!(bg, bg.clone());
}

#[test]
fn test_background_builder_failure() {
    let background_result = BackgroundBuilder::new("Invalid Background").build();
    assert!(
        background_result.is_err(),
        "background build should have failed"
    );

    let proficiency = PresentedOption::Base(SkillType::Athletics);

    let background_result_no_prof = BackgroundBuilder::new("No Proficiencies Background")
        .add_personality_traits(vec!["trait1".to_string(), "trait2".to_string()])
        .add_bond("bond".to_string())
        .add_flaw("flaw".to_string())
        .add_ideal("ideal".to_string())
        .build();
    let no_prof_err =
        background_result_no_prof.expect_err("Expected error due to no proficiencies");
    assert_eq!(
        no_prof_err,
        BackgroundBuildError::EmptyProficiencies,
        "Expected EmptyProficiencies error"
    );

    let background_result_no_bond = BackgroundBuilder::new("No Bond Background")
        .add_personality_traits(vec!["trait1".to_string(), "trait2".to_string()])
        .add_flaw("flaw".to_string())
        .add_ideal("ideal".to_string())
        .add_proficiency(proficiency.clone())
        .build();
    let no_bond_err = background_result_no_bond.expect_err("Expected error due to no bond");
    assert_eq!(
        no_bond_err,
        BackgroundBuildError::EmptyBonds,
        "Expected EmptyBond error"
    );

    let background_result_no_flaw = BackgroundBuilder::new("No Flaw Background")
        .add_personality_traits(vec!["trait1".to_string(), "trait2".to_string()])
        .add_bond("bond".to_string())
        .add_ideal("ideal".to_string())
        .add_proficiency(proficiency.clone())
        .build();
    let no_flaw_err = background_result_no_flaw.expect_err("Expected error due to no flaw");
    assert_eq!(
        no_flaw_err,
        BackgroundBuildError::EmptyFlaws,
        "Expected EmptyFlaw error"
    );

    let background_result_no_ideal = BackgroundBuilder::new("No Ideal Background")
        .add_personality_traits(vec!["trait1".to_string(), "trait2".to_string()])
        .add_bond("bond".to_string())
        .add_flaw("flaw".to_string())
        .add_proficiency(proficiency.clone())
        .build();
    let no_ideal_err = background_result_no_ideal.expect_err("Expected error due to no ideal");
    assert_eq!(
        no_ideal_err,
        BackgroundBuildError::EmptyIdeals,
        "Expected EmptyIdeal error"
    );

    let background_result_no_traits = BackgroundBuilder::new("No Traits Background")
        .add_bond("bond".to_string())
        .add_flaw("flaw".to_string())
        .add_ideal("ideal".to_string())
        .add_proficiency(proficiency)
        .build();
    let no_traits_err =
        background_result_no_traits.expect_err("Expected error due to no personality traits");
    assert_eq!(
        no_traits_err,
        BackgroundBuildError::NotEnoughPersonalityTraits,
        "Expected EmptyPersonalityTraits error"
    );

    let background_result_one_trait = BackgroundBuilder::new("One Trait Background")
        .add_personality_trait("Only one trait".to_string())
        .add_bond("bond".to_string())
        .add_flaw("flaw".to_string())
        .add_ideal("ideal".to_string())
        .add_proficiency(PresentedOption::Base(SkillType::Arcana))
        .build();
    let one_trait_err =
        background_result_one_trait.expect_err("Expected error due to only one personality trait");
    assert_eq!(
        one_trait_err,
        BackgroundBuildError::NotEnoughPersonalityTraits,
        "Expected InsufficientPersonalityTraits error"
    );
}

#[test]
fn bonds_and_flaws_preserved() {
    let background = BackgroundBuilder::new("Test Background")
        .add_proficiency(PresentedOption::Base(SkillType::Athletics))
        .add_personality_traits(vec!["Trait one".to_string(), "Trait two".to_string()])
        .add_ideal("Ideal".to_string())
        .add_bond("Bond one".to_string())
        .add_bond("Bond two".to_string())
        .add_flaw("Flaw one".to_string())
        .add_flaw("Flaw two".to_string())
        .build()
        .expect("background should build");


    assert_eq!(background.proficiencies(), &vec![PresentedOption::Base(SkillType::Athletics)]);

    assert_eq!(
        background.personality_traits(),
        &vec!["Trait one".to_string(), "Trait two".to_string()]
    );

    assert_eq!(
        background.bonds(),
        &vec!["Bond one".to_string(), "Bond two".to_string()]
    );
    assert_eq!(
        background.flaws(),
        &vec!["Flaw one".to_string(), "Flaw two".to_string()]
    );
}

#[test]
fn language_option_false_positives() {
    let mut language = LanguageOption::new_named_choice(vec![
        "Dwarvish".to_string(),
        "Halfling".to_string(),
    ]);

    assert!(!language.set_to("Elvish".to_string()));
    assert_eq!(
        language,
        LanguageOption::new_named_choice(vec![
            "Dwarvish".to_string(),
            "Halfling".to_string(),
        ])
    );
    assert!(language.set_to("dWaRvIsH".to_string()));
    assert_eq!(language, LanguageOption::new_fixed("Dwarvish".to_string()));
}
