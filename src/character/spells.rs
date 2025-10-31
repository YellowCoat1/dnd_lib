use std::str::FromStr;

use serde::{Serialize, Deserialize};
use super::{
    items::{Action, DamageRoll}, 
    stats::StatType
};

/// A spell definition, either manually created or loaded from the API.
///
/// `damage` represents spell damage scaling by level:
/// - Outer index = spell level (starting from base level)
/// - Inner index = multiple damage rolls per level (e.g., multi-damage spells like Chromatic Orb)// so spell.damage.unwrap()\[0\] returns it's regular damage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    /// The name of the spell, e.g. "Fireball"
    pub name: String,
    /// The description, split by paragraph
    pub description: Vec<String>,
    /// Textual description of higher level casting
    pub higher_level: Vec<String>,
    pub ritual: bool,
    pub concentration: bool,
    /// Casting time (e.g. "1 minute" "1 action")
    pub casting_time: String,
    /// Duration (e.g. "Instantaneous", "10 minutes")
    pub duration: String,
    /// Spell level (0 for cantrips)
    pub level: usize,
    /// Range (e.g. "60 feet")
    pub range: String,
    /// The school of magic that the spell is in
    pub school: School,
    /// The components of the spell, represened as characters. V, S, M are 'V', 'S', and 'M'.
    ///
    /// If D&D only supported these, it would be simplier to use a struct of bools, but there are
    /// others that pop up occasionally (Like the R, or "Royalty" component.)
    pub components: Vec<char>,
    /// If the spell has a material (M) component, it's listed here. (e.g. "a tiny bell and a piece of fine silver wire")
    pub material: Option<String>,
    /// if there is damage, this shows it for each of the levels. There also may be multiple
    /// different types, like chromatic orb's multiple damage types.
    pub damage: Option<Vec<Vec<DamageRoll>>>,
    /// Cantrips can have damage that scales with class level, rather than spell level. This
    /// represents that.
    ///
    /// The first field of the vec is the class level, and the second field is the damage.
    ///
    pub leveled_damage: Option<Vec<(usize, DamageRoll)>>
}

/// Represents a resolved spell's damage.
///
/// This is used when the attack roll/damage roll are already decided, and the spell is ready to
/// attack with.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellAction {
    pub name: String,
    pub spell_level: isize,
    pub damage_roll: DamageRoll,
    pub spell_attack_mod: isize,
}

impl Action for SpellAction {
    fn name(&self) -> &str { &self.name }
    fn damage_roll(&self) -> DamageRoll { self.damage_roll }
    fn attack_bonus(&self) -> isize { self.spell_attack_mod }
    fn damage_roll_bonus(&self) -> isize { 0 } // spells don't typically have a damage roll bonus
}

/// A school of magic.
/// 
/// Doc comments are just copy-pasted from the official descriptions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum School {
    ///Abjuration spells are protective in nature, though some of them have aggressive uses. They create magical barriers, negate harmful effects, harm trespassers, or banish creatures to other planes of existence.
    Abjuration,
    /// Conjuration spells involve the transportation of objects and creatures from one location to another. Some spells summon creatures or objects to the caster's side, whereas others allow the caster to teleport to another location. Some conjurations create objects or effects out of nothing.
    Conjuration,
    /// Divination spells reveal information, whether in the form of secrets long forgotten, glimpses of the future, the locations of hidden things, the truth behind illusions, or visions of distant people or places
    Divination,
    /// Enchantment spells affect the minds of others, influencing or controlling their behavior. Such spells can make enemies see the caster as a friend, force creatures to take a course of action, or even control another creature like a puppet.
    Enchantment,
    /// Evocation spells manipulate magical energy to produce a desired effect. Some call up blasts of fire or lightning. Others channel positive energy to heal wounds.
    Evocation,
    /// Illusion spells deceive the senses or minds of others. They cause people to see things that are not there, to miss things that are there, to hear phantom noises, or to remember things that never happened. Some illusions create phantom images that any creature can see, but the most insidious illusions plant an image directly in the mind of a creature
    Illusion,
    /// Necromancy spells manipulate the energies of life and death. Such spells can grant an extra reserve of life force, drain the life energy from another creature, create the undead, or even bring the dead back to life.
    Necromancy,
    /// Transmutation spells change the properties of a creature, object, or environment. They might turn an enemy into a harmless creature, bolster the strength of an ally, make an object move at the caster's command, or enhance a creature's innate healing abilities to rapidly recover from injury.
    Transmutation,
}


impl FromStr for School {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "abjuration" => Ok(School::Abjuration),
            "conjuration" => Ok(School::Conjuration),
            "divination" => Ok(School::Divination),
            "enchantment" => Ok(School::Enchantment),
            "evocation" => Ok(School::Evocation),
            "illusion" => Ok(School::Illusion),
            "necromancy" => Ok(School::Necromancy),
            "transmutation" => Ok(School::Transmutation),
            _ => Err(()),
        }
    }
}


/// Represents the spell slots for levels 0-9.
///
/// 0th levels is cantrips, so instead of spell slots, it's cantrips know.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpellSlots(pub [usize; 9]);

/// Represents pact magic spell slots.
///
/// In D&D, a warlock's pact magic behaves differently than typical spell slots, and when
/// multiclassing they are counted differently. For typical spell slots, see [SpellSlots]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PactSlots {
    pub num: usize,
    pub level: usize,
}

impl From<(usize, usize)> for PactSlots {
    fn from(value: (usize, usize)) -> Self {
        PactSlots {
            num: value.0,
            level: value.1,
        }
    }
}

/// Spellcasting data for a class, including slots, ability, and spell lists.
///
/// Cantrips but not spell slots are included, since cantrips are class-wide and spell slots are
/// charachter-wide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spellcasting {
    /// How many cantrips for every level
    pub cantrips_per_level: [usize; 20],
    /// The ability type used for spellcasting (e.g. Intelligence, Wisdom, Charisma)
    pub spellcasting_ability: StatType,
    /// The list of spells availible for each spell level (1-9)
    pub spell_list: [Vec<String>; 10],
    /// Cantrips availible per level
    pub spellcaster_type: SpellCasterType,
    /// If the caster knows or prepares their spells.
    pub preperation_type: SpellCastingPreperation,
}

/// Type of spellcaster (full caster, half caster, quarter-caster)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpellCasterType {
    Full,
    Half,
    Quarter,
    /// Warlocks get a special case, since they have a seperate spell slots list.
    Warlock,
}

/// How the spellcaster prepares their spell list.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpellCastingPreperation {
    /// Spells are prepared after every long rest from the spell list. Wizards are included in
    /// this, and just prepare spells from their spell book instead of spell list.
    ///
    /// The amount of spells that can be prepared is always the spellcasting ability modifier + the
    /// class level.
    Prepared,
    /// Spells are innately known, and cannot be swapped on long rest. Bards and Sorcerers fit into
    /// this.
    Known,
}

/// The spell slots for every level of a full spell caster.
pub const CASTER_SLOTS: [[usize; 9]; 20] = [
    [2, 0, 0, 0, 0, 0, 0, 0, 0],
    [3, 0, 0, 0, 0, 0, 0, 0, 0],
    [4, 2, 0, 0, 0, 0, 0, 0, 0],
    [4, 3, 0, 0, 0, 0, 0, 0, 0],
    [4, 3, 2, 0, 0, 0, 0, 0, 0],
    [4, 3, 3, 0, 0, 0, 0, 0, 0],
    [4, 3, 3, 1, 0, 0, 0, 0, 0],
    [4, 3, 3, 2, 0, 0, 0, 0, 0],
    [4, 3, 3, 3, 1, 0, 0, 0, 0],
    [4, 3, 3, 3, 2, 0, 0, 0, 0],
    [4, 3, 3, 3, 2, 1, 0, 0, 0],
    [4, 3, 3, 3, 2, 1, 0, 0, 0],
    [4, 3, 3, 3, 2, 1, 1, 0, 0],
    [4, 3, 3, 3, 2, 1, 1, 0, 0],
    [4, 3, 3, 3, 2, 1, 1, 1, 0],
    [4, 3, 3, 3, 2, 1, 1, 1, 0],
    [4, 3, 3, 3, 2, 1, 1, 1, 1],
    [4, 3, 3, 3, 3, 1, 1, 1, 1],
    [4, 3, 3, 3, 3, 2, 1, 1, 1],
    [4, 3, 3, 3, 3, 2, 2, 1, 1],
];

/// Warlock pact casting.
///
/// Lists by level. The first in the tuple is the amount of spell slots, and the second is the spell
/// level.
pub const PACT_CASTING_SLOTS: [(usize, usize); 20] = [
    (1, 1),
    (2, 1),
    (2, 2),
    (2, 2),
    (2, 3),
    (2, 3),
    (2, 4),
    (2, 4),
    (2, 5),
    (2, 5),
    (3, 5),
    (3, 5),
    (3, 5),
    (3, 5),
    (3, 5),
    (3, 5),
    (4, 5),
    (4, 5),
    (4, 5),
    (4, 5),
];
