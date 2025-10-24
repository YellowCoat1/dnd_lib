use std::collections::HashMap;
use super::features::{PresentedOption, Feature};
use super::stats::{EquipmentProficiencies, SkillType, StatType};
use super::spells::Spellcasting;
use super::items::{Item, WeaponType, ArmorCategory};
use serde::{Serialize, Deserialize};

/// A D&D Class.
///
/// This is a static class that contains all the information needed for a character to take it. For
/// a class in application, see [SpeccedClass](super::character::SpeccedClass) instead.
///
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub subclasses: Vec<Subclass>,
    /// features are listed by level.
    pub features: Vec<Vec<PresentedOption<Feature>>>,
    pub beginning_items: Vec<PresentedOption<Vec<(ItemCategory, usize)>>>,
    pub saving_throw_proficiencies: Vec<StatType>,
    /// The dice size of a hit die, e.g. 12 is 1d12.
    pub hit_die: usize,
    /// The first field of the tuple is how many to choose, so 3 is choose 3 skills.
    pub skill_proficiency_choices: (usize,PresentedOption<SkillType>),
    pub spellcasting: Option<Spellcasting>,
    pub equipment_proficiencies: EquipmentProficiencies,
    /// The features that appear on a class's table, rather than text features.
    /// They're indexed by name, and returns the values for all 20 levels.
    pub class_specific_leveled: HashMap<String, [String; 20]>,
}

impl Class {
    /// gets the class's features up until a specific level.
    /// this returns every feature a class would have at the specified level
    pub fn get_all_features_at_level(&self, level: usize) -> Vec<&PresentedOption<Feature>> {
        self.features[0..level]
            .iter()
            .flatten()
            .collect()
    }

    /// getting the class's features at a specific level.
    /// this returns only the features that are gained at this level, not features before that.
    pub fn get_specific_features_at_level(&self, level: usize) -> &Vec<PresentedOption<Feature>> {
        self.features.get(level-1).expect("Couldn't get class features at level!")
    }
}

/// A D&D Subclass.
///
/// Subclasses are contained within classes.
/// To add a subclass, push it to the [Class]'s subclasses field.
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Subclass {
    pub name: String,
    pub description: Vec<String>,
    /// Features are per-level, for each of the 20 levels.
    /// Most of these will be empty vectors, since subclasses don't give features every level.
    pub features: [Vec<PresentedOption<Feature>>; 20],
}

/// Category of an item, used for class item lists.
///
/// E.g. "A longbow", "Light armor", "A simple weapon".
/// Can be filled in by the user into the base ([ItemCategory::Item]) type.
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum ItemCategory {
    Item(Item),
    Weapon(WeaponType),
    Armor(ArmorCategory),
}

#[cfg(test)]
mod tests {
    use super::*;

    // tests the "get all features at level" and "get specific features at level"
    #[test]
    fn get_correct_features() {

        let feature_1 = Feature {
            name: "feature1".to_string(),
            description: vec![],
            effects: vec![],
        };

        let feature_2 = Feature {
            name: "feature2".to_string(),
            description: vec![],
            effects: vec![],
        };

        let features = vec![
            vec![PresentedOption::Base(feature_1.clone())],
            vec![PresentedOption::Base(feature_2.clone())],
        ];
        
        let test_class = Class {
            name: "test class".to_string(),
            subclasses: vec![],
            features,
            beginning_items: vec![],
            saving_throw_proficiencies: vec![],
            hit_die: 4,
            skill_proficiency_choices: (0, PresentedOption::Base(SkillType::Investigation)),
            equipment_proficiencies: EquipmentProficiencies::default(),
            spellcasting: None,
            class_specific_leveled: HashMap::new(),
        };

        let error_msg: &str = "failed to get correct class features";

        // 1st and 2nd level should individually only have 1st and 2nd level features, respectively
        assert_eq!(test_class.get_specific_features_at_level(1), &vec![PresentedOption::Base(feature_1.clone())], "{error_msg}");
        assert_eq!(test_class.get_specific_features_at_level(2), &vec![PresentedOption::Base(feature_2.clone())], "{error_msg}");

        // 1st level total should only have 1st level features. 2nd level total should have both.
        assert_eq!(test_class.get_all_features_at_level(1), vec![&PresentedOption::Base(feature_1.clone())], "{error_msg}");
        assert_eq!(test_class.get_all_features_at_level(2), vec![&PresentedOption::Base(feature_1.clone()), &PresentedOption::Base(feature_2.clone())], "{error_msg}");
    }
}
