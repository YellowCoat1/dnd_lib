/// Features, or etc listed effects. 
use serde::{Serialize, Deserialize};
use super::{items::{ArmorCategory, WeaponType}, stats::{SkillType, StatType}};
pub use super::choice::{PresentedOption, chosen};

/// A feature represents a general effect/trait. Any extra effect from a race, class, etc is a feature.
///
/// e.g. Darkvision, extra attack, ability score increase
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Feature {
    pub name: String,
    pub description: Vec<String>,
    pub effects: Vec<FeatureEffect>,
}

// features are keyed by name, so equal features have equal names
impl PartialEq for Feature {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }

    fn ne(&self, other: &Self) -> bool {
        self.name != other.name
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub enum AbilityScoreIncrease {
    StatIncrease(Option<StatType>, Option<StatType>),
    AddedFeature(Option<Feature>), // This can be filled in by the user, idfk
    Unchosen,
}

/// Different effects a feature can have.
/// This list will grow as the crate is developed.
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub enum FeatureEffect {
    AddSaveProficiency(StatType),
    AddSaveModifier(StatType, isize),
    /// Basic ability score modifier
    AddModifier(StatType, isize),
    AddSkillProficiency(SkillType),
    WeaponProficiency(WeaponType),
    ArmorProficiency(ArmorCategory),
    EtcProficiency(String),
    AddSkillModifier(SkillType, isize),
    ACBonus(isize),
    AbilityScoreIncrease(AbilityScoreIncrease),
    UnarmoredDefense(isize, StatType, Option<StatType>),
    Expertise([Option<SkillType>; 2]),
    /// Hp increases by 1 for every level
    LeveledHpIncrease,
}

