#![cfg(feature = "network-intensive-tests")]
use dnd_lib::character::spells::SpellSlots;
use dnd_lib::character::spells::CASTER_SLOTS;
use dnd_lib::prelude::*;

use futures::future::try_join_all;

#[tokio::test]
async fn level_3_druid() {
    let provider = Dnd5eapigetter::new();
    let human_future = provider.get_race("human");
    let druid_future = provider.get_class("druid");
    let acolyte_future = provider.get_background("acolyte");

    let spells = vec![
        // cantrips
        provider.get_spell("poison spray"),
        provider.get_spell("shillelagh"),
        // 1st level
        provider.get_spell("charm person"),
        provider.get_spell("cure wounds"),
        provider.get_spell("thunderwave"),
        provider.get_spell("healing word"),
        // 2nd level
        provider.get_spell("moonbeam"),
        provider.get_spell("darkvision"),
    ];

    let human = human_future.await.expect("couldn't get human");
    let druid = druid_future.await.expect("couldnt't get druid");
    let acolyte = acolyte_future.await.expect("couldn't get acolyte");

    let stats = Stats {
        strength: 8,
        constitution: 13,
        dexterity: 14,
        intelligence: 12,
        wisdom: 15,
        charisma: 10,
    };

    let mut boopo = Character::new("Boopo".to_string(), &druid, &acolyte, &human, stats);

    // choose skill proficiencies granted by the class

    boopo
        .class_skill_proficiencies
        .get_mut(0)
        .expect("Character should have a 1st choice for skill proficiencies")
        // this is the 6th choice, which is perception
        .choose_in_place(5);

    boopo
        .class_skill_proficiencies
        .get_mut(1)
        .expect("Character should have a 2nd choice for skill proficiencies")
        // this is the 8th choice, which is Survival
        .choose_in_place(7);

    boopo.level_up_to_level(&druid, 3);

    // choose subclass
    boopo.classes[0].subclass.choose_in_place(0);

    assert_eq!(boopo.spell_slots(), Some(SpellSlots(CASTER_SLOTS[2])));
    assert_eq!(
        boopo.available_spell_slots,
        Some(SpellSlots(CASTER_SLOTS[2]))
    );
    assert_eq!(boopo.available_pact_slots, None);

    let v = boopo.prepare_spells_multiple();
    assert_eq!(
        v.len(),
        1,
        "There were more classes returned by the spells prepared utility than there should be"
    );
    let (_, prepped_spell_list, amount, cantrips) = v.into_iter().next().unwrap();
    assert_eq!(amount, 6, "incorrect number of spells to prepare");
    assert_eq!(cantrips, 2, "incorrect number of cantrips to prepare");
    *prepped_spell_list = try_join_all(spells).await.expect("Couldn't get spells");

    let spells = boopo.classes[0]
        .spellcasting
        .as_ref()
        .expect("Druid should be a spellcaster")
        .1
        .iter()
        .map(|spell| spell.name.to_lowercase())
        .collect::<Vec<_>>();
    let spells_lower = spells.iter().map(|v| v.to_lowercase()).collect::<Vec<_>>();
    assert_eq!(
        spells_lower,
        vec![
            "poison spray",
            "shillelagh",
            "charm person",
            "cure wounds",
            "thunderwave",
            "healing word",
            "moonbeam",
            "darkvision"
        ]
    );

    let spell_attacks = boopo.spell_actions();
    // macro to help test spell attack damage
    macro_rules! assert_spell_damage {
        ($char:expr, $spell_index:expr, $expected_damage:expr) => {
            let spell_attack = spell_attacks.get($spell_index);
            match spell_attack {
                Some(attack) => {
                    assert_eq!(
                        attack.damage_roll.to_string(),
                        $expected_damage,
                        "Damage roll for spell at index '{}' did not match expected value",
                        $spell_index
                    );
                }
                None => panic!("Spell attack at index '{}' not found", $spell_index),
            }
        };
    }

    assert_spell_damage!(boopo, 0, "1d12 Poison"); // Poison Spray
    assert_spell_damage!(boopo, 1, "2d8 Thunder"); // Thunderwave (1st level)
    assert_spell_damage!(boopo, 2, "3d8 Thunder"); // Thunderwave (2nd level)
    assert_spell_damage!(boopo, 3, "2d10 Radiant"); // Moonbeam

    let poison_spray = spell_attacks
        .first()
        .expect("Couldn't get poison spray spell attack");
    let moonbeam = spell_attacks
        .get(3)
        .expect("Couldn't get moonbeam spell attack");

    boopo.cast(poison_spray, None);
    assert_eq!(
        boopo.available_spell_slots,
        Some(SpellSlots([4, 2, 0, 0, 0, 0, 0, 0, 0])),
        "Spell slots after casting poison spray did not match expected value"
    );
    boopo.cast(moonbeam, None);
    assert_eq!(
        boopo.available_spell_slots,
        Some(SpellSlots([4, 1, 0, 0, 0, 0, 0, 0, 0])),
        "Spell slots after casting moonbeam did not match expected value"
    );

    boopo.long_rest();
    assert_eq!(
        boopo.available_spell_slots,
        Some(SpellSlots([4, 2, 0, 0, 0, 0, 0, 0, 0])),
        "Spell slots after long rest did not match expected value"
    );
}
