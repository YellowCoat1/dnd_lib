/// Features, or etc listed effects. 
use serde::{Serialize, Deserialize};
use super::{
    items::{ArmorCategory, WeaponType}, 
    stats::{SkillType, StatType}
};

pub use super::choice::{PresentedOption, chosen};

/// A feature represents a general effect/trait. Any extra effect from a race, class, etc is a feature.
///
/// e.g. Darkvision, extra attack, ability score increase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    // The name of the feature
    pub name: String,
    /// The description, split by paragraph 
    pub description: Vec<String>,
    /// The mechanical effects that the feature causes.
    pub effects: Vec<FeatureEffect>,
}

// features are keyed by name, so equal features have equal names
impl PartialEq for Feature {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// An ability score increase for a character, usually granted at certain class levels.
///
/// Players can choose to:
/// - Increase two ability scores by +1 each
/// - Increase one ability score by +2
/// - Optionally, take a bonus feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityScoreIncrease {
    /// Increase ability scores. A `None` variant means that it's still unchosen.
    StatIncrease(Option<StatType>, Option<StatType>),
    /// Instead of taking a score increase, taking a feature instead.
    ///
    /// Since there's no reasonable way for this library to hold every feature you can take, this
    /// an open ended option that you can fill with any feature you choose.
    AddedFeature(Option<Feature>), 
    Unchosen,
}

/// Different effects a feature can have.
///
/// This list will grow as the crate is developed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureEffect {
    /// Grants proficiency in a saving throw
    AddSaveProficiency(StatType),
    /// Adds a bonus to a saving throw
    AddSaveModifier(StatType, isize),
    /// Adds a flat modifier to an ability score
    AddModifier(StatType, isize),
    /// Gives proficiency in a weapon type
    WeaponProficiency(WeaponType),
    /// Gives proficiency in an armor type
    ArmorProficiency(ArmorCategory),
    /// Gives proficiency in an etc tool or weapon
    EtcProficiency(String),
    /// Gives proficiency in a skill
    AddSkillProficiency(SkillType),
    /// Adds a flat modifier to a specific skill
    AddSkillModifier(SkillType, isize),
    /// Gives a flat bonus to AC
    ACBonus(isize),
    /// An ability score increase
    AbilityScoreIncrease(AbilityScoreIncrease),
    /// Grants unarmored defense. 
    ///
    /// The first is the base, which an ability score modifier is added
    /// onto, and then optionally another ability score modifier is added on.
    ///
    /// E.g. Barbarian unarmored defense is 8+DEX+CON, which here is (8, Dex, Some(Con))
    UnarmoredDefense(isize, StatType, Option<StatType>),
    /// Grants expertise (adding proficiency a second time) in up to two different skills.
    Expertise([Option<SkillType>; 2]),
    /// Adds +1 HP for every character level
    LeveledHpIncrease,

    UnarmoredMovement,
    SpeedBonus(usize),
}
