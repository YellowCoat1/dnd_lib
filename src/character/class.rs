use super::features::{Feature, PresentedOption};
use super::items::{ArmorCategory, Item, WeaponType};
use super::spells::Spellcasting;
use super::stats::{EquipmentProficiencies, SkillType, StatType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// unarmored movement
pub(crate) const UNARMORED_MOVEMENT: [usize; 20] = [
    0, 10, 10, 10, 10, 15, 15, 15, 15, 20, 20, 20, 20, 25, 25, 25, 25, 30, 30, 30,
];

/// A D&D Class.
///
/// This is a static class that contains all the information needed for a character to take it. For
/// a class in application, see [SpeccedClass](crate::character::SpeccedClass) instead.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Class {
    pub name: String,
    pub subclasses: Vec<Subclass>,
    /// features are listed by level.
    pub features: [Vec<PresentedOption<Feature>>; 20],
    pub beginning_items: Vec<PresentedOption<Vec<(ItemCategory, usize)>>>,
    pub saving_throw_proficiencies: Vec<StatType>,
    /// The dice size of a hit die, e.g. 12 is 1d12.
    pub hit_die: usize,
    /// The first field of the tuple is how many to choose.
    pub skill_proficiency_choices: (usize, PresentedOption<SkillType>),
    /// If the class is a spellcaster. Contains all the relevant information for spellcasting.
    pub spellcasting: Option<Spellcasting>,
    pub equipment_proficiencies: EquipmentProficiencies,
    /// The features that appear on a class's table, rather than text features. =
    /// They're indexed by name, and returns the values for all 20 levels.
    pub class_specific_leveled: HashMap<String, [String; 20]>,

    /// The prerequisites for multiclassing into this class. By default, these are "and"ed together.
    pub multiclassing_prerequisites: HashMap<StatType, usize>,
    /// If true, the prerequisites are "or"ed together rather than "and"ed.
    pub multiclassing_prerequisites_or: bool,
    /// The proficiencies a character would gain if they multiclassed into this class.
    pub multiclassing_proficiency_gain: EquipmentProficiencies,

    /// See [TrackedField] for more information.
    pub tracked_fields: Vec<TrackedField>,
}

impl Class {
    /// gets the class's features up until a specific level.
    /// this returns every feature a class would have at the specified level
    pub fn get_all_features_at_level(&self, level: usize) -> Vec<&PresentedOption<Feature>> {
        self.features[0..level].iter().flatten().collect()
    }

    /// getting the class's features at a specific level.
    /// this returns only the features that are gained at this level, not features before that.
    pub fn get_specific_features_at_level(&self, level: usize) -> &Vec<PresentedOption<Feature>> {
        self.features
            .get(level - 1)
            .expect("Couldn't get class features at level!")
    }
}

/// A D&D Subclass.
///
/// Subclasses are contained within [Classes](Class).
/// To add a subclass, push it to the [Class]'s subclasses field.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Subclass {
    pub name: String,
    /// The subclass's description, split by paragraph.
    pub description: Vec<String>,
    /// Features are per-level, for each of the 20 levels.
    /// Most of these will be empty vectors, since subclasses don't give features every level.
    pub features: [Vec<PresentedOption<Feature>>; 20],
}

/// Category of an item, used for class item lists.
///
/// E.g. "A longbow", "Light armor", "A simple weapon".
/// Can be filled in by the user into the base ([ItemCategory::Item]) type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemCategory {
    Item(Item),
    Weapon(WeaponType),
    Armor(ArmorCategory),
}

/// Tracks a resource that the class uses. Things like the barbarian rages or the druid wildshapes,
/// which need to be actively tracked and stored.
///
/// This stores only the metadata about the field; the actual value for a character would be stored
/// in that character's [SpeccedClass](crate::character::SpeccedClass).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackedField {
    pub name: String,
    /// restore on long rest
    pub long_rest: bool,
    /// restore on short rest
    pub short_rest: bool,
    /// restore on level up
    pub level_up: bool,
    /// a class specific field that acts as the maximum.
    ///
    /// For example, the class specific field of "rage count" for barbarian at level 5 is 3, and so
    /// with a Some("rage count") for this field, the max would be 3 if they're level 5.
    pub class_specific_max: Option<String>,
    /// A hard set maximum. If both class_specific_max and hard_max are set, then hard_max takes
    /// precedent.
    pub hard_max: Option<usize>,
}

impl TrackedField {
    /// Get the maximum at level 1. Useful for getting the beginning value
    pub fn get_base_max(&self, class: &Class) -> Option<usize> {
        let level_1_fields: HashMap<&String, &String> = class
            .class_specific_leveled
            .iter()
            .map(|(k, v)| (k, &v[0]))
            .collect();
        self.hard_max.or(self
            .class_specific_max
            .clone()
            .and_then(|v| level_1_fields.get(&v)?.parse().ok()))
    }
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

        let mut features: [Vec<PresentedOption<Feature>>; 20] = Default::default();
        features[0] = vec![PresentedOption::Base(feature_1.clone())];
        features[1] = vec![PresentedOption::Base(feature_2.clone())];

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
            multiclassing_prerequisites: HashMap::new(),
            multiclassing_prerequisites_or: false,
            multiclassing_proficiency_gain: EquipmentProficiencies::default(),
            etc_fields: vec![],
        };

        let error_msg: &str = "failed to get correct class features";

        // 1st and 2nd level should individually only have 1st and 2nd level features, respectively
        assert_eq!(
            test_class.get_specific_features_at_level(1),
            &vec![PresentedOption::Base(feature_1.clone())],
            "{error_msg}"
        );
        assert_eq!(
            test_class.get_specific_features_at_level(2),
            &vec![PresentedOption::Base(feature_2.clone())],
            "{error_msg}"
        );

        // 1st level total should only have 1st level features. 2nd level total should have both.
        assert_eq!(
            test_class.get_all_features_at_level(1),
            vec![&PresentedOption::Base(feature_1.clone())],
            "{error_msg}"
        );
        assert_eq!(
            test_class.get_all_features_at_level(2),
            vec![
                &PresentedOption::Base(feature_1.clone()),
                &PresentedOption::Base(feature_2.clone())
            ],
            "{error_msg}"
        );
    }
}
