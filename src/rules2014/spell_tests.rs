use std::str::FromStr;

use crate::rules2014::spells::{School, SpellSlots};

#[test]
fn spell_slot_eq() {
    let spell_slots_1 = SpellSlots([3, 1, 1, 0, 0, 0, 0, 0, 0]);
    let spell_slots_2 = SpellSlots([3, 1, 1, 0, 0, 0, 0, 0, 0]);
    let spell_slots_3 = SpellSlots([3, 2, 1, 0, 0, 0, 0, 0, 0]);

    assert_eq!(spell_slots_1, spell_slots_2, "Spell slots equality false negative");
    assert_ne!(spell_slots_1, spell_slots_3, "Spell slots equality false positive");
    assert_ne!(spell_slots_2, spell_slots_3, "Spell slots equality false positive");
}

#[test]
fn spell_school_strings() {
    use School::*;
    let spell_schools = [
        Abjuration, 
        Conjuration, 
        Divination, 
        Enchantment, 
        Evocation, 
        Illusion, 
        Necromancy, 
        Transmutation
    ];
    let errored_school = spell_schools.iter()
        .find(|v| Some(**v) != School::from_str(&v.to_string()).ok());

    if let Some(s) = errored_school {
        panic!("Spell school {}", s);
    }

}
