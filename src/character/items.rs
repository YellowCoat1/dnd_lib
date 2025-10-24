//! D&D items, item types, and damage types.
use std::cmp::PartialEq;

use serde::{Serialize, Deserialize};

use super::features::Feature;

#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
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

impl DamageType {
    pub fn from_string(name: &str) -> Result<DamageType, ()> {
        match name.to_lowercase().as_str() {
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
#[derive(Debug)]
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub enum ItemType {
    Weapon(Weapon),
    Armor(Armor),
    Shield,
    Misc,
}

/// A single item. 
///
/// Often, items with counts are stored as a (Item, usize) tuple.
#[derive(Clone)]
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub description: Option<String>,
    pub item_type: ItemType,
    pub features: Vec<Feature>,
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct Armor {
    pub ac: isize,
    pub category: ArmorCategory,
    pub strength_minimum: Option<usize>,
    pub stealth_disadvantage: bool,
}

impl Armor {
    pub fn get_ac(&self, dex: isize) -> isize {
        self.ac + match self.category {
            ArmorCategory::Light => dex,
            ArmorCategory::Medium => if dex > 2 {2} else {dex},
            ArmorCategory::Heavy => 0,
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum ArmorCategory {
    Light,
    Medium,
    Heavy,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Weapon {
    pub damage: DamageRoll,
    pub attack_roll_bonus: usize,
    pub weapon_type: WeaponType,
    pub properties: WeaponProperties,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(Default)]
#[derive(Serialize, Deserialize)]
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
    pub versitile: Option<DamageRoll>,
}


#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum WeaponType {
    Simple,
    SimpleRanged,
    Martial,
    MartialRanged,
}


/// A damage roll in the format AdB xyz damage, 
/// e.g. 1d6 piercing.
#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct DamageRoll {
    pub number: usize, 
    pub dice: usize, 
    pub damage_type: DamageType
}

pub trait Action {
    fn name(&self) -> &String;
    fn attack_bonus(&self) -> isize;
    fn damage_roll(&self) -> DamageRoll;
    fn damage_roll_bonus(&self) -> isize;
}

pub struct WeaponAction {
    pub name: String,
    pub attack_bonus: isize,
    pub damage_roll: DamageRoll,
    pub damage_roll_bonus: isize,
    pub two_handed: bool,
    pub second_attack: bool,
}

impl Action for WeaponAction {
    fn name(&self) -> &String {
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

    /// Construct a damage roll from a string in the form of XdY. for example, 2d10.
    pub fn from_str(s: &str, damage_type: DamageType) -> Option<DamageRoll> {
        let p: Vec<&str> = s.split('d').collect();
        if p.len() == 2 {
            if let (Ok(a), Ok(b)) = (p[0].parse::<usize>(), p[1].parse::<usize>()) {
                return Some(DamageRoll::new(a, b, damage_type));
            }
        }
        None
    }
}
