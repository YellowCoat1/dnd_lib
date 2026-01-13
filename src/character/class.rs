use super::features::{Feature, PresentedOption};
use super::items::{ArmorCategory, Item, WeaponType};
use super::spells::Spellcasting;
use super::stats::{EquipmentProficiencies, SkillType, StatType};
use heck::ToTitleCase;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// unarmored movement
pub(crate) const UNARMORED_MOVEMENT: [usize; 20] = [
    0, 10, 10, 10, 10, 15, 15, 15, 15, 20, 20, 20, 20, 25, 25, 25, 25, 30, 30, 30,
];

/// A D&D Class.
///
/// This is a static class that contains all the information needed for a character to take it. For
/// a class in application, see [SpeccedClass](crate::character::player_character::SpeccedClass) instead.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Class {
    name: String,
    pub subclasses: Vec<Subclass>,
    features: [Vec<PresentedOption<Feature>>; 20],
    beginning_items: Vec<PresentedOption<Vec<(ItemCategory, usize)>>>,
    saving_throw_proficiencies: Vec<StatType>,
    hit_die: usize,
    skill_proficiency_choices: (usize, PresentedOption<SkillType>),
    spellcasting: Option<Spellcasting>,
    equipment_proficiencies: EquipmentProficiencies,
    class_specific_leveled: HashMap<String, [String; 20]>,
    multiclassing_prerequisites: HashMap<StatType, usize>,
    multiclassing_prerequisites_or: bool,
    multiclassing_proficiency_gain: EquipmentProficiencies,
    tracked_fields: Vec<TrackedField>,
}

impl Class {
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Gets the class's features.
    ///
    /// features are listed by level.
    pub fn features(&self) -> &[Vec<PresentedOption<Feature>>; 20] {
        &self.features
    }
    /// Gets the list of beginning possible items for the class.
    ///
    /// This defines a list of options, where each option is a list of ([ItemCategory], quantity) tuples.
    ///
    /// The items are stored as an ItemCategory, which can be a specific item, or a category like
    /// "simple weapon" or "light armor". The second field of the tuple is the quantity of that
    /// item/category.
    ///
    /// So, for instance, "choose between a longbow with 20 arrows or a simple weapon"
    /// would be represented as two options: a [(longbow,1), (arrow, 20)], and a [(simple weapon, 1)].
    pub fn beginning_items(&self) -> &Vec<PresentedOption<Vec<(ItemCategory, usize)>>> {
        &self.beginning_items
    }
    /// The list of saving throw proficiencies granted by the class. Just about every class has
    /// this as two stats.
    pub fn saving_throw_proficiencies(&self) -> &Vec<StatType> {
        &self.saving_throw_proficiencies
    }
    /// Gets the dice size of a hit die, e.g. 12 is 1d12.
    pub fn hit_die(&self) -> usize {
        self.hit_die
    }
    /// Gets the skill proficiency choices for the class.
    ///
    /// The first field of the tuple is how many to choose, and the second is the options.
    /// e.g. (2, [Athletics, Acrobatics, Stealth]) means "choose 2 from Athletics, Acrobatics, and
    /// Stealth".
    pub fn skill_proficiency_choices(&self) -> &(usize, PresentedOption<SkillType>) {
        &self.skill_proficiency_choices
    }
    /// If the class is a spellcaster. Contains all the relevant information for spellcasting.
    pub fn spellcasting(&self) -> Option<&Spellcasting> {
        self.spellcasting.as_ref()
    }
    /// The equipment proficiencies granted by the class, e.g. "light armor", "simple weapons",
    /// "land vehicles"
    pub fn equipment_proficiencies(&self) -> &EquipmentProficiencies {
        &self.equipment_proficiencies
    }
    /// The features that appear on a class's table, rather than text features. =
    /// They're indexed by name, and returns the values for all 20 levels.
    pub fn class_specific_leveled(&self) -> &HashMap<String, [String; 20]> {
        &self.class_specific_leveled
    }
    /// The prerequisites for multiclassing into this class. By default, these are "and"ed together.
    ///
    /// e.g. (Strength: 13, Dexterity: 13) means "You must have at least 13 Strength and 13
    /// Dexterity to multiclass into this class".
    pub fn multiclassing_prerequisites(&self) -> &HashMap<StatType, usize> {
        &self.multiclassing_prerequisites
    }
    /// If true, the prerequisites are "or"ed together rather than "and"ed.
    ///
    /// e.g. [Class::multiclassing_prerequisites] as (Strength: 13, Dexterity: 13) with this set to true means "You must have at least 13
    /// Strength or 13 Dexterity to multiclass into this class".
    ///
    /// By default, this is false, meaning the prerequisites are "and"ed together.
    pub fn multiclassing_prerequisites_or(&self) -> bool {
        self.multiclassing_prerequisites_or
    }
    /// The proficiencies a character would gain if they multiclassed into this class.
    pub fn multiclassing_proficiency_gain(&self) -> &EquipmentProficiencies {
        &self.multiclassing_proficiency_gain
    }
    /// Fields that must be actively tracked by the character. See [TrackedField] for more information.
    pub fn tracked_fields(&self) -> &Vec<TrackedField> {
        &self.tracked_fields
    }

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

impl std::fmt::Display for ItemCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemCategory::Item(item) => write!(f, "a {}", item.name),
            ItemCategory::Weapon(weapon_type) => write!(f, "any {} weapon", weapon_type),
            ItemCategory::Armor(armor_category) => write!(f, "any {} armor", armor_category.to_string().to_lowercase()),
        }
    }
}

/// Tracks a resource that the class uses. Things like the barbarian rages or the druid wildshapes,
/// which need to be actively tracked and stored.
///
/// This stores only the metadata about the field; the actual value for a character would be stored
/// in that character's [SpeccedClass](crate::character::player_character::SpeccedClass).
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

/// A builder for [Class].
///
/// The following fields are required before building:
/// - name
/// - features
/// - hit_die
/// - skill_proficiency_choices
///  
pub struct ClassBuilder {
    name: Option<String>,
    subclasses: Vec<Subclass>,
    features: Option<[Vec<PresentedOption<Feature>>; 20]>,
    beginning_items: Vec<PresentedOption<Vec<(ItemCategory, usize)>>>,
    saving_throw_proficiencies: Vec<StatType>,
    hit_die: Option<usize>,
    skill_proficiency_choices: Option<(usize, PresentedOption<SkillType>)>,
    equipment_proficiencies: EquipmentProficiencies,
    spellcasting: Option<Spellcasting>,
    class_specific_leveled: HashMap<String, [String; 20]>,
    multiclassing_prerequisites: HashMap<StatType, usize>,
    multiclassing_prerequisites_or: bool,
    multiclassing_proficiency_gain: EquipmentProficiencies,
    tracked_fields: Vec<TrackedField>,
}

impl ClassBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            subclasses: vec![],
            features: None,
            beginning_items: vec![],
            saving_throw_proficiencies: vec![],
            hit_die: None,
            skill_proficiency_choices: None,
            equipment_proficiencies: EquipmentProficiencies::default(),
            spellcasting: None,
            class_specific_leveled: HashMap::new(),
            multiclassing_prerequisites: HashMap::new(),
            multiclassing_prerequisites_or: false,
            multiclassing_proficiency_gain: EquipmentProficiencies::default(),
            tracked_fields: vec![],
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name.to_title_case());
        self
    }

    /// Adds a subclass to the list of subclasses.
    pub fn add_subclass(mut self, subclass: Subclass) -> Self {
        self.subclasses.push(subclass);
        self
    }

    /// Sets the subclasses to the provided list.
    pub fn add_subclasses<T>(mut self, subclasses: T) -> Self
    where
        T: IntoIterator<Item = Subclass>,
    {
        self.subclasses.extend(subclasses);
        self
    }

    pub fn set_features(mut self, features: [Vec<PresentedOption<Feature>>; 20]) -> Self {
        self.features = Some(features);
        self
    }

    /// Adds a beginning item to the list of beginning items.
    pub fn add_beginning_item(mut self, item: PresentedOption<Vec<(ItemCategory, usize)>>) -> Self {
        self.beginning_items.push(item);
        self
    }

    /// Adds multiple beginning items to the list of beginning items.
    pub fn add_beginning_items<T>(mut self, items: T) -> Self
    where
        T: IntoIterator<Item = PresentedOption<Vec<(ItemCategory, usize)>>>,
    {
        self.beginning_items.extend(items);
        self
    }

    pub fn add_saving_throw_proficiency(mut self, stat: StatType) -> Self {
        self.saving_throw_proficiencies.push(stat);
        self
    }

    pub fn add_multiple_save_proficiencies<T>(mut self, stats: T) -> Self
    where
        T: IntoIterator<Item = StatType>,
    {
        self.saving_throw_proficiencies.extend(stats);
        self
    }

    pub fn set_hit_die(mut self, hit_die: usize) -> Self {
        self.hit_die = Some(hit_die);
        self
    }

    pub fn set_skill_proficiency_choices(
        mut self,
        num_choices: usize,
        options: Vec<SkillType>,
    ) -> Self {
        let choices = if options.len() == 1 {
            PresentedOption::Base(options[0])
        } else {
            PresentedOption::Choice(options)
        };
        self.skill_proficiency_choices = Some((num_choices, choices));
        self
    }

    pub fn add_equipment_proficiencies(mut self, proficiencies: EquipmentProficiencies) -> Self {
        self.equipment_proficiencies += proficiencies;
        self
    }

    pub fn set_spellcasting(mut self, spellcasting: Option<Spellcasting>) -> Self {
        self.spellcasting = spellcasting;
        self
    }

    pub fn add_class_specific_field(mut self, name: String, values: [String; 20]) -> Self {
        self.class_specific_leveled
            .insert(name.to_title_case(), values);
        self
    }

    pub fn add_class_specific_fields<T>(mut self, fields: T) -> Self
    where
        T: IntoIterator<Item = (String, [String; 20])>,
    {
        self.class_specific_leveled.extend(fields);
        self
    }

    pub fn add_multiclassing_prerequisite(mut self, stat: StatType, value: usize) -> Self {
        self.multiclassing_prerequisites.insert(stat, value);
        self
    }

    pub fn add_multiclassing_prerequisites<T>(mut self, prerequisites: T) -> Self
    where
        T: IntoIterator<Item = (StatType, usize)>,
    {
        self.multiclassing_prerequisites.extend(prerequisites);
        self
    }

    /// Sets whether the multiclassing prerequisites are "or"ed together, instead of "and"ed as
    /// usual.
    ///
    /// e.g. "either Strength 13 or Dexterity 13" instead of "Strength 13 and Dexterity 13".
    pub fn set_multiclassing_prerequisites_or(mut self, or: bool) -> Self {
        self.multiclassing_prerequisites_or = or;
        self
    }

    /// Adds proficiencies gained when multiclassing into this class.
    pub fn add_multiclassing_proficiency(mut self, proficiencies: EquipmentProficiencies) -> Self {
        self.multiclassing_proficiency_gain += proficiencies;
        self
    }

    pub fn add_tracked_field(mut self, field: TrackedField) -> Self {
        self.tracked_fields.push(field);
        self
    }

    pub fn add_tracked_fields<T>(mut self, fields: T) -> Self
    where
        T: IntoIterator<Item = TrackedField>,
    {
        self.tracked_fields.extend(fields);
        self
    }

    pub fn build(self) -> Result<Class, String> {
        Ok(Class {
            name: self.name.ok_or("Class name is required")?,
            subclasses: self.subclasses,
            features: self.features.ok_or("Class features are required")?,
            beginning_items: self.beginning_items,
            saving_throw_proficiencies: self.saving_throw_proficiencies,
            hit_die: self.hit_die.ok_or("Class hit die is required")?,
            skill_proficiency_choices: self
                .skill_proficiency_choices
                .ok_or("Class skill proficiency choices are required")?,
            equipment_proficiencies: self.equipment_proficiencies,
            spellcasting: self.spellcasting,
            class_specific_leveled: self.class_specific_leveled,
            multiclassing_prerequisites: self.multiclassing_prerequisites,
            multiclassing_prerequisites_or: self.multiclassing_prerequisites_or,
            multiclassing_proficiency_gain: self.multiclassing_proficiency_gain,
            tracked_fields: self.tracked_fields,
        })
    }
}

impl Default for ClassBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::character::items::ItemType;

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
            tracked_fields: vec![],
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

    #[test]
    fn item_formatting() {
        let longbow = ItemCategory::Item(Item {
            name: "Longbow".to_string(),
            description: None,
            features: vec![],
            item_type: ItemType::Misc,
        });
        let simple_weapon = ItemCategory::Weapon(WeaponType::Simple);
        let light_armor = ItemCategory::Armor(ArmorCategory::Light);

        assert_eq!(longbow.to_string(), "a Longbow");
        assert_eq!(simple_weapon.to_string(), "any simple weapon");
        assert_eq!(light_armor.to_string(), "any light armor");
    }
}
