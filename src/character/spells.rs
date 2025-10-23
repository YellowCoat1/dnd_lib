use serde::{Serialize, Deserialize};
use super::{items::DamageRoll, stats::StatType};

/// A spell.
/// 
/// Constructed manually or from the api.
///
/// The damage vector starts at the current level, then counts up, 
/// so spell.damage.unwrap()\[0\] returns it's regular damage.
#[derive(Serialize, Deserialize)]
pub struct Spell {
    pub name: String,
    pub description: Vec<String>,
    pub higher_level: Vec<String>,
    pub ritual: bool,
    pub concentration: bool,
    pub casting_time: String,
    pub duration: String,
    pub level: usize,
    pub range: String,
    pub school: School,
    pub components: Vec<char>,
    pub material: Option<String>,
    pub damage: Option<Vec<DamageRoll>>,
}

/// A school of magic.
#[derive(Serialize, Deserialize)]
pub enum School {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,
}


impl School {
    pub fn from_string(s: &str) -> Result<School, ()> {
        let v = match s.to_lowercase().as_str() {
            "abjuration" => School::Abjuration,
            "conjuration" => School::Conjuration,
            "divination" => School::Divination,
            "enchantment" => School::Enchantment,
            "evocation" => School::Evocation,
            "illusion" => School::Illusion,
            "necromancy" => School::Necromancy,
            "transmutation" => School::Transmutation,
            _ => return Err(()),
        };
        Ok(v)
    }
}


#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct SpellSlots(pub usize, pub usize, pub usize, pub usize, pub usize, pub usize, pub usize, pub usize, pub usize, pub usize);

/// All the info necessary for a spellcasting class
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Spellcasting {
    pub spell_slots_per_level: Vec<SpellSlots>,
    pub spellcasting_ability: StatType,
    pub spell_list: [Vec<String>; 10]
}


