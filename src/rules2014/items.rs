//! D&D items, item types, and damage types.
use std::cmp::PartialEq;

use serde::{Deserialize, Serialize};

use super::{features::Feature, stats::EquipmentProficiencies};
use strum::{Display as StrumDisplay, EnumString};

/// Lists the names of all regular D&D simple weapons. Ranged weapons are listed seperately.
///
/// If you want an [Item], you need to use a get function, like
/// [Dnd5eapigetter](crate::prelude::Dnd5eapigetter).
pub const SIMPLE_WEAPONS_MELEE: [&str; 9] = [
    "club",
    "dagger",
    "greatclub",
    "handaxe",
    "javelin",
    "mace",
    "quarterstaff",
    "sickle",
    "spear",
];

/// Lists the names of all regular D&D ranged simple weapons.
///
/// See [SIMPLE_WEAPONS_MELEE].
///
/// If you want an [Item], you need to use a get function, like
/// [Dnd5eapigetter](crate::prelude::Dnd5eapigetter).
pub const SIMPLE_WEAPONS_RANGED: [&str; 4] = ["crossbow light ", "dart", "shortbow", "sling"];

/// Lists the names of all regular D&D simple weapons.
///
/// Uses [SIMPLE_WEAPONS_MELEE] and [SIMPLE_WEAPONS_RANGED].
pub const fn simple_weapons() -> [&'static str; 13] {
    let mut weapons = [""; 13];
    let mut i = 0;
    while i < SIMPLE_WEAPONS_MELEE.len() {
        weapons[i] = SIMPLE_WEAPONS_MELEE[i];
        i += 1;
    }
    let mut j = 0;
    while j < SIMPLE_WEAPONS_RANGED.len() {
        weapons[i + j] = SIMPLE_WEAPONS_RANGED[j];
        j += 1;
    }
    weapons
}

/// Lists the names of all regular D&D martial weapons.
///
/// Uses [MARTIAL_WEAPONS_MELEE] and [MARTIAL_WEAPONS_RANGED].
pub const fn martial_weapons() -> [&'static str; 23] {
    let mut weapons = [""; 23];
    let mut i = 0;
    while i < MARTIAL_WEAPONS_MELEE.len() {
        weapons[i] = MARTIAL_WEAPONS_MELEE[i];
        i += 1;
    }
    let mut j = 0;
    while j < MARTIAL_WEAPONS_RANGED.len() {
        weapons[i + j] = MARTIAL_WEAPONS_RANGED[j];
        j += 1;
    }
    weapons
}

/// Lists the names of all regular D&D melee martial weapons. Ranged weapons are listed seperately.
///
/// If you want an [Item], you need to use a get function, like
/// [Dnd5eapigetter](crate::prelude::Dnd5eapigetter).
pub const MARTIAL_WEAPONS_MELEE: [&str; 18] = [
    "battleaxe",
    "flail",
    "glaive",
    "greataxe",
    "greatsword",
    "halberd",
    "lance",
    "longsword",
    "maul",
    "morningstar",
    "pike",
    "rapier",
    "scimitar",
    "shortsword",
    "trident",
    "war pick",
    "warhammer",
    "whip",
];

/// Lists the names of all regular D&D ranged martial weapons.
///
/// See [MARTIAL_WEAPONS_MELEE].
///
/// If you want an [Item], you need to use a get function, like
/// [Dnd5eapigetter](crate::prelude::Dnd5eapigetter).
pub const MARTIAL_WEAPONS_RANGED: [&str; 5] = [
    "blowgun",
    "crossbow hand",
    "crossbow heavy",
    "longbow",
    "net",
];

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, StrumDisplay,
)]
#[strum(ascii_case_insensitive)]
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

/// A general type an item could be.
///
/// Shields are distinct from Armor, since they're calculated differently, and you may only receive
/// the bonus of one armor piece at a time.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Weapon(Weapon),
    Armor(Armor),
    Shield,
    Misc,
}

/// A single item.
///
/// If you want to be able to store multiple of the same item, use [ItemCount].
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

/// An item along with a count of how many of that item there are.
/// For example, 20 arrows, or 1 potion of healing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCount {
    pub item: Item,
    pub count: usize,
}

impl From<Item> for ItemCount {
    fn from(item: Item) -> Self {
        ItemCount { item, count: 1 }
    }
}
impl From<(Item, usize)> for ItemCount {
    fn from((item, count): (Item, usize)) -> Self {
        ItemCount { item, count }
    }
}

/// A character's armor.
///
/// Note that this doesn't include shields.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Armor {
    pub ac: isize,
    pub category: ArmorCategory,
    pub strength_minimum: Option<usize>,
    pub stealth_disadvantage: bool,
}

impl Armor {
    /// Get the ac of the armor in the context of a character using it.
    pub fn total_ac(&self, dex: isize) -> isize {
        self.ac
            + match self.category {
                ArmorCategory::Light => dex,
                ArmorCategory::Medium => dex.min(2),
                ArmorCategory::Heavy => 0,
            }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    StrumDisplay,
    EnumString,
)]
/// The different categories for armor.
#[strum(ascii_case_insensitive)]
pub enum ArmorCategory {
    /// Light armor, e.g. leather. Dexterity bonus gets added to the ac.
    Light,
    // Medium armor, e.g. Scale Mail. Dexterity bonus, up to 2, gets added to the ac.
    Medium,
    // Heavy armor, e.g. Plate. Dexterity bonus does not get added to the ac, though they have the
    // highest base ACs.
    Heavy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Weapon {
    /// The damage the weapon causes on hit.
    pub damage: DamageRoll,
    /// A flat attack roll bonus added before proficiencies or stats. E.g. a +2 greatsword would
    /// have 2, but a regular greatsword would have 0.
    pub attack_roll_bonus: usize,
    pub weapon_type: WeaponType,
    pub properties: WeaponProperties,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// If the weapon is versatile, this contains the damage roll of the 2 handed attack.
    pub versatile: Option<DamageRoll>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    StrumDisplay,
    EnumString,
)]
#[strum(serialize_all = "lowercase")]
pub enum WeaponType {
    Simple,
    SimpleRanged,
    Martial,
    MartialRanged,
}

///
/// Takes equipment proficiencies and a weapon type, returns if the proficiencies has that weapon
/// type.
pub fn is_proficient_with(weapon: &WeaponType, proficiencies: &EquipmentProficiencies) -> bool {
    matches!(
        (
            proficiencies.simple_weapons,
            proficiencies.martial_weapons,
            weapon
        ),
        (_, true, WeaponType::Martial)
            | (_, true, WeaponType::MartialRanged)
            | (true, _, WeaponType::Simple)
            | (true, _, WeaponType::SimpleRanged)
    )
}

/// A damage roll in the format XdY (type) damage,
/// e.g. 1d6 piercing.
///
/// This doesn't also store added damage, e.g. 1d6+2. If you want to store that, use a (DamageRoll,
///  isize)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DamageRoll {
    /// The number of dice rolled
    pub number: usize,
    /// The numer of faces in the die (e.g. 4, 8, 20)
    pub dice: usize,
    /// the constant bonus added to the damage roll
    pub bonus: isize,
    /// The type of damage the roll causes.
    pub damage_type: DamageType,
}

impl std::fmt::Display for DamageRoll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}d{} {}", self.number, self.dice, self.damage_type)
    }
}

/// An action that a character could take.
///
/// This only covers damage-dealing actions, like a shortsword attack or a magic missle, and not
/// etc actions, like a push.
pub trait Action {
    fn name(&self) -> &str;
    fn attack_bonus(&self) -> isize;
    fn damage_roll(&self) -> DamageRoll;
}

/// An attack you can take with a weapon.
///
/// This is after calculations, so a WeaponAction has a static attack roll bonus and damage roll
/// type that each include the extra bonuses from ability scores.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WeaponAction {
    pub name: String,
    pub attack_bonus: isize,
    pub damage_roll: DamageRoll,
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
}

impl DamageRoll {
    pub fn new(number: usize, dice: usize, bonus: isize, damage_type: DamageType) -> DamageRoll {
        DamageRoll {
            number,
            dice,
            damage_type,
            bonus,
        }
    }

    /// Parses a string of the form "XdY" into a DamageRoll.
    ///
    /// For example, "2d10" would be turned into a DamageRoll with 2 dice and 10 faces.
    pub fn from_str(s: &str, damage_type: DamageType) -> Option<DamageRoll> {
        let (a, b) = s.split_once('d')?;
        let number = a.parse().ok()?;
        let dice;
        let bonus;
        if b.contains('+') || b.contains('-') {
            let (c, d) = b.split_once(['+', '-'])?;
            dice = c.parse().ok()?;
            bonus = d.parse().ok()?;
        } else {
            dice = b.parse().ok()?;
            bonus = 0;
        }

        Some(Self {
            number,
            dice,
            bonus,
            damage_type,
        })
    }
}

/// An item that a character is holding, along with whether or not it's equipped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldEquipment {
    pub item: Item,
    pub quantity: usize,
    pub equipped: bool,
}

impl HeldEquipment {
    pub fn new(item: Item, quantity: usize, equipped: bool) -> HeldEquipment {
        HeldEquipment {
            item,
            quantity,
            equipped,
        }
    }

    pub fn equip(&mut self) {
        self.equipped = true;
    }

    pub fn unequip(&mut self) {
        self.equipped = false;
    }
}

impl From<Item> for HeldEquipment {
    fn from(item: Item) -> Self {
        HeldEquipment {
            item,
            quantity: 1,
            equipped: false,
        }
    }
}

impl From<ItemCount> for HeldEquipment {
    fn from(item_count: ItemCount) -> Self {
        HeldEquipment {
            item: item_count.item,
            quantity: item_count.count,
            equipped: false,
        }
    }
}

impl From<(Item, usize)> for HeldEquipment {
    fn from((item, quantity): (Item, usize)) -> Self {
        HeldEquipment {
            item,
            quantity,
            equipped: false,
        }
    }
}
impl From<(Item, usize, bool)> for HeldEquipment {
    fn from((item, quantity, equipped): (Item, usize, bool)) -> Self {
        HeldEquipment {
            item,
            quantity,
            equipped,
        }
    }
}
impl From<HeldEquipment> for ItemCount {
    fn from(held: HeldEquipment) -> Self {
        ItemCount {
            item: held.item,
            count: held.quantity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_weapons() {
        let simple = simple_weapons();
        assert_eq!(simple.len(), 13);
        for weapon in SIMPLE_WEAPONS_MELEE.iter() {
            assert!(simple.contains(weapon));
        }
        for weapon in SIMPLE_WEAPONS_RANGED.iter() {
            assert!(simple.contains(weapon));
        }

        let martial = martial_weapons();
        assert_eq!(martial.len(), 23);
        for weapon in MARTIAL_WEAPONS_MELEE.iter() {
            assert!(martial.contains(weapon));
        }
        for weapon in MARTIAL_WEAPONS_RANGED.iter() {
            assert!(martial.contains(weapon));
        }
    }

    #[test]
    fn conversions() {
        let base_item = Item {
            name: "Test Item".to_string(),
            description: None,
            item_type: ItemType::Misc,
            features: vec![],
        };
        let item_count = ItemCount::from(base_item.clone());
        assert_eq!(item_count.count, 1);
        assert_eq!(item_count.item, base_item);
    }

    #[test]
    fn armor() {
        let plate = Armor {
            ac: 18,
            category: ArmorCategory::Heavy,
            strength_minimum: Some(15),
            stealth_disadvantage: true,
        };
        assert_eq!(plate.total_ac(4), 18);
        assert_eq!(plate.total_ac(0), 18);
        let splint = Armor {
            ac: 17,
            category: ArmorCategory::Medium,
            strength_minimum: Some(13),
            stealth_disadvantage: true,
        };
        assert_eq!(splint.total_ac(4), 19);
        assert_eq!(splint.total_ac(2), 19);
        assert_eq!(splint.total_ac(0), 17);
    }

    #[test]
    fn proficient_with() {
        let result = is_proficient_with(
            &WeaponType::Martial,
            &EquipmentProficiencies {
                simple_weapons: true,
                martial_weapons: false,
                ..Default::default()
            },
        );
        assert!(!result);

        let result = is_proficient_with(
            &WeaponType::SimpleRanged,
            &EquipmentProficiencies {
                simple_weapons: true,
                martial_weapons: false,
                ..Default::default()
            },
        );
        assert!(result);
    }

    #[test]
    fn weapon_actions() {
        let action = WeaponAction {
            name: "Longsword Attack".to_string(),
            attack_bonus: 5,
            damage_roll: DamageRoll {
                number: 1,
                dice: 8,
                bonus: 3,
                damage_type: DamageType::Slashing,
            },
            two_handed: false,
            second_attack: false,
        };

        assert_eq!(action.name(), "Longsword Attack");
        assert_eq!(action.attack_bonus(), 5);
        assert_eq!(
            action.damage_roll(),
            DamageRoll {
                number: 1,
                dice: 8,
                bonus: 3,
                damage_type: DamageType::Slashing,
            }
        );
    }

    #[test]
    fn held_equipment() {
        let base_item = Item {
            name: "Shield".to_string(),
            description: None,
            item_type: ItemType::Shield,
            features: vec![],
        };

        let mut held = HeldEquipment::from((base_item.clone(), 1, false));
        assert_eq!(held.item, base_item);
        assert_eq!(held.quantity, 1);
        assert!(!held.equipped);

        held.equip();
        assert!(held.equipped);

        held.unequip();
        assert!(!held.equipped);
        let held_other = HeldEquipment::new(base_item.clone(), 2, true);
        assert_eq!(held_other.item, base_item);
        let held_other_2 = HeldEquipment::from((base_item.clone(), 2));
        assert_eq!(held_other_2.item, base_item);
        assert_eq!(held_other_2.quantity, 2);
        let held_other_3 = HeldEquipment::from(base_item);
        assert_eq!(held_other_3.item.name, "Shield");
    }
}
