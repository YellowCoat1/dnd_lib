use std::str::FromStr;

use crate::rules2014::{items::{Action, DamageRoll, DamageType}, spells::{PactSlots, School, SpellAction, SpellSlots}};

#[test]
fn spell_slot_eq() {
    let spell_slots_1 = SpellSlots([3, 1, 1, 0, 0, 0, 0, 0, 0]);
    let spell_slots_2 = SpellSlots([3, 1, 1, 0, 0, 0, 0, 0, 0]);
    let spell_slots_3 = SpellSlots([3, 2, 1, 0, 0, 0, 0, 0, 0]);

    assert_eq!(spell_slots_1, spell_slots_2, "Spell slots equality false negative");
    assert_ne!(spell_slots_1, spell_slots_3, "Spell slots equality false positive");
    assert_ne!(spell_slots_2, spell_slots_3, "Spell slots equality false positive");


    assert!(spell_slots_3 > spell_slots_2);
    assert!(spell_slots_2 < spell_slots_3);
    assert!(spell_slots_1 == spell_slots_2);
    assert_eq!(spell_slots_1.partial_cmp(&spell_slots_2), Some(std::cmp::Ordering::Equal));


    assert_eq!(PactSlots::default(), PactSlots {num: 1, level: 1});
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

#[test]
fn spell_actions() {
    let spell_action = SpellAction {
        name: String::from("action"),
        spell_level: 1,
        damage_roll: DamageRoll::new(1, 20, 0, DamageType::Bludgeoning),
        spell_attack_mod: 3,
    };

    assert_eq!(spell_action.name(), "action");
    assert_eq!(spell_action.damage_roll(), DamageRoll::new(1, 20, 0, DamageType::Bludgeoning));
    assert_eq!(spell_action.attack_bonus(), 3);

    let spell_action_2 = SpellAction {
        spell_level: 2,
        ..spell_action.clone()
    };
    assert_ne!(spell_action, spell_action_2);



}
