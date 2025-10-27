//! D&D items, item types, and damage types.
use std::{cmp::PartialEq, str::FromStr};

use serde::{Serialize, Deserialize};

use super::{features::Feature, stats::EquipmentProficiencies};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DamageType {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
}

impl FromStr for DamageType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "acid" => Ok(DamageType::Acid),
            "bludgeoning" => Ok(DamageType::Bludgeoning),
            "cold" => Ok(DamageType::Cold),
            "fire" => Ok(DamageType::Fire),
            "force" => Ok(DamageType::Force),
            "lightning" => Ok(DamageType::Lightning),
            "necrotic" => Ok(DamageType::Necrotic),
            "piercing" => Ok(DamageType::Piercing),
            "poison" => Ok(DamageType::Poison),
            "psychic" => Ok(DamageType::Psychic),
            "radiant" => Ok(DamageType::Radiant),
            "slashing" => Ok(DamageType::Slashing),
            "thunder" => Ok(DamageType::Thunder),
            _ => Err(())
        }
    }
}

/// A general type an item could be.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon(Weapon),
    Armor(Armor),
    Shield,
    Misc,
}

/// A single item. 
///
/// Often, items with counts are stored as a (Item, usize) tuple.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// What type of item this is (weapon, armor, etc).
    pub item_type: ItemType,
    /// Any extra features/effects this item grants
    pub features: Vec<Feature>,
}

/// A character's armor.
///
/// Note that this doesn't include shields, they have their own kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Armor {
    pub ac: isize,
    pub category: ArmorCategory,
    pub strength_minimum: Option<usize>,
    pub stealth_disadvantage: bool,
}

impl Armor {
    /// Get the ac of the armor if you used it.
    pub fn total_ac(&self, dex: isize) -> isize {
        self.ac + match self.category {
            ArmorCategory::Light => dex,
            ArmorCategory::Medium => dex.min(2),
            ArmorCategory::Heavy => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// The different categories for armor.
pub enum ArmorCategory {
    /// Light armor, e.g. leather. Dexterity bonus gets added to the ac.
    Light,
    // Medium armor, e.g. Scale Mail. Dexterity bonus, up to 2, gets added to the ac.
    Medium,
    // Heavy armor, e.g. Plate. Dexterity bonus does not get added to the ac, though they have the
    // highest base ACs.
    Heavy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Weapon {
    /// The damage the weapon causes on hit.
    pub damage: DamageRoll,
    /// A flat attack roll bonus added before proficiencies or stats. E.g. a +2 greatsword would
    /// have 2, but a regular greatsword would have 0.
    pub attack_roll_bonus: usize,
    pub weapon_type: WeaponType,
    pub properties: WeaponProperties,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WeaponProperties {
    pub ammunition: bool,
    pub finesse: bool,
    pub heavy: bool,
    pub light: bool,
    pub loading: bool,
    pub monk: bool,
    pub reach: bool,
    pub special: bool,
    pub thrown: bool,
    pub two_handed: bool,
    pub versatile: Option<DamageRoll>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WeaponType {
    Simple,
    SimpleRanged,
    Martial,
    MartialRanged,
}

/// Takes equipment proficiencies and a weapon type, returns if the proficiencies has that weapon
/// type.
pub fn is_proficient_with(weapon: &WeaponType, proficiencies: &EquipmentProficiencies) -> bool {
    matches!(
        (proficiencies.simple_weapons, proficiencies.martial_weapons, weapon), 
        (_, true, WeaponType::Martial) | 
        (_, true, WeaponType::MartialRanged) | 
        (true, _, WeaponType::Simple) | 
        (true, _, WeaponType::SimpleRanged)
    )
}


/// A damage roll in the format XdY (type) damage, 
/// e.g. 1d6 piercing.
///
/// This doesn't also store added damage, e.g. 1d6+2. If you want to store that, use a (DamageRoll, 
///  isize)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DamageRoll {
    /// The number of dice rolled
    pub number: usize, 
    /// The numer of faces in the die (e.g. 4, 8, 20)
    pub dice: usize, 
    /// The type of damage the roll causes.
    pub damage_type: DamageType
}

/// An action that a character could take.
///
/// This only covers damage-dealing actions, like a shortsword attack or a magic missle, and not
/// etc actions, like a push.
pub trait Action {
    fn name(&self) -> &str;
    fn attack_bonus(&self) -> isize;
    fn damage_roll(&self) -> DamageRoll;
    fn damage_roll_bonus(&self) -> isize;
}

/// An attack you can take with a weapon.
///
/// This is after calculations, so a WeaponAction has a static attack roll bonus and damage roll
/// type.
#[derive(Debug, Clone)]
pub struct WeaponAction {
    pub name: String,
    pub attack_bonus: isize,
    pub damage_roll: DamageRoll,
    pub damage_roll_bonus: isize,
    pub two_handed: bool,
    pub second_attack: bool,
}

impl Action for WeaponAction {
    fn name(&self) -> &str {
        &self.name
    }
    fn attack_bonus(&self) -> isize {
        self.attack_bonus
    }
    fn damage_roll(&self) -> DamageRoll {
        self.damage_roll
    }
    fn damage_roll_bonus(&self) -> isize {
        self.damage_roll_bonus
    }
}

impl DamageRoll {
    pub fn new(number: usize, dice: usize, damage_type: DamageType) -> DamageRoll {
        DamageRoll { 
            number, 
            dice, 
            damage_type,
        }
    }

    /// Parses a string of the form "XdY" into a DamageRoll.
    /// 
    /// For example, "2d10" would be turned into a DamageRoll with 2 dice and 10 faces.
    pub fn from_str(s: &str, damage_type: DamageType) -> Option<DamageRoll> {
        let (a, b) = s.split_once('d')?;
        Some(Self{
            number: a.parse().ok()?,
            dice: b.parse().ok()?,
            damage_type,
        })
    }
}
