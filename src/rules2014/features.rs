//! Features, or etc listed effects.
//!
//! Most character effects that are descriptive rather than numerical are listed under here.
//! As of right now, that only includes the [Feature] struct, and the many surrounding
//! datastructures around it.
//!
//! Another major part of this module is the [PresentedOption] enum, which is used throughout the
//! crate to describe a choice between options that the character can pick.

use super::background::LanguageOption;

use super::{
    items::{Action, ArmorCategory, DamageRoll, WeaponType},
    stats::{SkillType, StatType},
};
use serde::{Deserialize, Serialize};

pub use super::choice::*;

/// A feature represents a general effect/trait. Any extra effect from a race, class, etc is a feature.
///
/// e.g. Darkvision, extra attack, or an ability score increase.
///
/// Features are used in this crate to represent any non-numeric effect that a character can have.
/// These effects are unpredictable and varied, so Features need to be flexible to represent them.
///
/// All features have a name. Most have a description, which lists what the feature does. For
/// select features, there are mechanical effects that this crate supports, which are listed under
/// [Feature::effects]. Each [FeatureEffect] represents a mechanical effect that the feature has on
/// the character. See [FeatureEffect] for more details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    // The name of the feature
    pub name: String,
    /// The description, split by paragraph
    pub description: Vec<String>,
    /// The mechanical effects that the feature causes.
    pub effects: Vec<FeatureEffect>,
}

/// An ability score increase for a character, usually granted at certain class levels.
///
/// Players can choose to:
/// - Increase two ability scores by +1 each
/// - Increase one ability score by +2
/// - Optionally, take a bonus feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl AbilityScoreIncrease {
    pub fn set_stat_increase(&mut self, first: StatType, second: Option<StatType>) {
        *self = AbilityScoreIncrease::StatIncrease(Some(first), second);
    }
}

/// An action granted by a feature.
///
/// This is meant to be a wildcard action, describing any attack that isn't already in the domain of
/// this crate. Its fields reflect this, covering every possibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAction {
    pub name: String,
    /// A number that is always added to the attack roll
    pub static_attack_bonus: usize,
    /// Stats that are added to the attack roll
    pub attack_bonus_stats: Vec<StatType>,
    /// If proficiency is added to the attack roll
    pub add_prof_to_attack: bool,
    /// the base damage roll and type
    pub damage_roll: DamageRoll,
    /// Stats that are always added to the damage roll
    pub damage_bonus_stats: Vec<StatType>,
    /// If proficiency is added to the damage
    pub add_prof_to_damage: bool,
}

impl PartialEq for CustomAction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.damage_roll == other.damage_roll
    }
}

/// A CustomAction after its fields have been computed within a character.
///
/// This struct has everything needed to make an attack.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComputedCustomAction {
    pub name: String,
    pub attack_bonus: isize,
    pub damage_roll: DamageRoll,
}

impl Action for ComputedCustomAction {
    fn name(&self) -> &str {
        &self.name
    }

    fn damage_roll(&self) -> DamageRoll {
        self.damage_roll
    }
    fn attack_bonus(&self) -> isize {
        self.attack_bonus
    }
}

/// Different mechanical effects a [Feature] can have.
///
/// Features describe any effect something may have on a character. Some of these effects have
/// mechanical implications that this crate can represent. These mechanical effects are listed
/// here.
///
/// This list will grow as the crate is developed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeatureEffect {
    /// Grants proficiency in a saving throw
    AddSaveProficiency(StatType),
    /// Adds a bonus to a saving throw.
    AddSaveModifier(StatType, isize),
    /// Adds a flat modifier to an ability score. This is capped at 20.
    AddModifier(StatType, isize),
    /// Adds a flat modifier to an ability score. This is uncapped.
    AddModifierUncapped(StatType, isize),
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

    /// Implements monk unarmored movement
    /// Shouldn't be added outside of monk, as it depends on monk level.
    UnarmoredMovement,
    /// Adds a flat bonus to your speed
    SpeedBonus(usize),
    /// Adds a flying speed to the character
    FlyingSpeed(usize),
    /// Adds a hovering speed to the character
    HoveringSpeed(usize),
    /// Adds a burrowing speed to the character
    BurrowingSpeed(usize),
    /// Adds a climbing speed to the character
    ClimbingSpeed(usize),
    /// Adds a swimming speed to the character
    SwimmingSpeed(usize),

    /// An extra damage roll added by a feature. It doesn't need to be a damage roll, it can just
    /// be an extra damage (e.g. bonus 1d6 poison damage on melee attack)
    CustomAction(CustomAction),

    /// Grants an extra language
    AddedLanguage(LanguageOption),
}

#[cfg(test)]
mod tests {
    use crate::rules2014::{
        features::CustomAction,
        items::{DamageRoll, DamageType},
    };

    fn null_custom_action() -> CustomAction {
        CustomAction {
            name: "Null Action".to_string(),
            static_attack_bonus: 0,
            attack_bonus_stats: vec![],
            add_prof_to_attack: false,
            damage_roll: DamageRoll {
                number: 0,
                dice: 0,
                bonus: 0,
                damage_type: DamageType::Cold,
            },
            damage_bonus_stats: vec![],
            add_prof_to_damage: false,
        }
    }

    fn null_computed_action() -> super::ComputedCustomAction {
        super::ComputedCustomAction {
            name: "Null Action".to_string(),
            attack_bonus: 0,
            damage_roll: DamageRoll {
                number: 0,
                dice: 0,
                bonus: 0,
                damage_type: DamageType::Cold,
            },
        }
    }

    #[test]
    fn custom_actions() {
        use super::Action;
        let action1 = null_custom_action();
        let mut action2 = null_custom_action();
        assert_eq!(action1, action2);
        action2.name = "Different Name".to_string();
        assert_ne!(action1, action2);

        let computed1 = null_computed_action();
        let mut computed2 = null_computed_action();
        assert_eq!(computed1, computed2);
        computed2.name = "Different Name".to_string();
        assert_ne!(computed1, computed2);

        assert_eq!(computed1.name(), "Null Action");
        assert_eq!(computed1.attack_bonus(), 0);
        assert_eq!(computed1.damage_roll(), action1.damage_roll);
    }
}
