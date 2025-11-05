#[cfg(feature = "network-intensive-tests")]
use dnd_lib::prelude::*;
#[cfg(feature = "network-intensive-tests")]
use dnd_lib::character::spells::PactSlots;
#[cfg(feature = "network-intensive-tests")]
use dnd_lib::character::stats::StatType;
#[cfg(feature = "network-intensive-tests")]
use dnd_lib::character::features::FeatureEffect;

#[cfg(feature = "network-intensive-tests")]
use futures::future::try_join_all;

#[cfg(feature = "network-intensive-tests")]
#[tokio::test]
async fn level_10_warlock() {
    let provider = Dnd5eapigetter::new();
    let tiefling_future = provider.get_race("tiefling");
    let warlock_future = provider.get_class("warlock");
    let acolyte_future = provider.get_background("acolyte");


    let spells = vec![
        // cantrips
        provider.get_spell("eldritch blast"),
        provider.get_spell("minor illusion"),
        // 1st level
        provider.get_spell("hellish rebuke"),
        provider.get_spell("charm person"),
        // 2nd level
        provider.get_spell("darkness"),
        provider.get_spell("invisibility"),
        // 3rd level
        provider.get_spell("counterspell"),
        provider.get_spell("dispel magic"),
        // 4th level
        provider.get_spell("blight"),
        provider.get_spell("banishment"),
        // 5th level
        provider.get_spell("hold monster"),
        provider.get_spell("dream"),
    ];


    let tiefling = tiefling_future.await.expect("couldn't get tiefling");
    let warlock = warlock_future.await.expect("couldnt't get warlock");
    let acolyte = acolyte_future.await.expect("couldn't get acolyte");

    let stats = Stats {
        strength: 8,
        constitution: 14,
        dexterity: 13,
        intelligence: 12,
        wisdom: 10,
        charisma: 15,
    };

    let mut baroopa = Character::new("Baroopa".to_string(), &warlock, &acolyte, &tiefling, stats);

    // choose skill proficiencies granted by the class

    // deception
    baroopa.class_skill_proficiencies
        .get_mut(0)
        .expect("Character should have a 1st choice for skill proficiencies")
        .choose_in_place(1);
    // Investigation
    baroopa.class_skill_proficiencies
        .get_mut(1)
        .expect("Character should have a 2nd choice for skill proficiencies")
        .choose_in_place(4);


    baroopa.level_up_to_level(&warlock, 10); 

    // choose subclass
    // this is the fiend patron
    baroopa.classes[0].subclass.choose_in_place(0);


    // find all ability score increases
    let ability_score_increases = baroopa.classes[0].current_class_features
        .iter_mut()
        .flat_map( |level_features|level_features.iter_mut())
        .filter_map(|v|  v.as_base_mut())
        .flat_map(|v| v.effects.iter_mut())
        .filter_map(|v| match v {
            FeatureEffect::AbilityScoreIncrease(a) => Some(a),
            _ => None,
        });


    let mut ability_score_increases_vec = ability_score_increases.collect::<Vec<_>>();
    assert_eq!(ability_score_increases_vec.len(), 2, "Warlock should have 2 ability score increases by level 10");
    ability_score_increases_vec[0]
        .set_stat_increase(StatType::Charisma, Some(StatType::Charisma));
    ability_score_increases_vec[1]
        .set_stat_increase(StatType::Charisma, Some(StatType::Dexterity));

    assert_eq!(baroopa.stats(), Stats::from(&[8, 14, 14, 13, 10, 20]));

    // There should be no spells to prepare for warlock, as they know their spells.
    assert_eq!(baroopa.prepare_spells().len(), 0);

    // add the spells
    let warlock_spells = &mut baroopa.classes[0].spellcasting
        .as_mut()
        .expect("Warlock should be a spellcaster")
       .1;
    *warlock_spells = try_join_all(spells).await
        .expect("Couldn't get spells");


    let warlock_spells = baroopa.classes[0].spellcasting
        .as_ref()
        .expect("Warlock should be a spellcaster")
        .1
        .iter()
        .map(|spell| spell.name.to_lowercase())
        .collect::<Vec<_>>();

    assert_eq!(warlock_spells, vec![
        "eldritch blast",
        "minor illusion",
        "hellish rebuke",
        "charm person",
        "darkness",
        "invisibility",
        "counterspell",
        "dispel magic",
        "blight",
        "banishment",
        "hold monster",
        "dream",
    ]);

    assert_eq!(baroopa.classes[0].spellcasting.as_ref().unwrap().1.len(), 12, "Warlock should have 12 spells known at level 10");

    let spell_actions = baroopa.spell_actions();
    assert_eq!(spell_actions.len(), 9, "Warlock should have 9 spell actions at level 10");
    assert_eq!(spell_actions[0].name.to_lowercase(), "eldritch blast", "First spell action should be eldritch blast");
    assert_eq!(spell_actions[0].damage_roll.to_string(), "1d10 Force", "Eldritch blast damage roll should be 1d10 Force");
    
    assert_eq!(spell_actions[1].name.to_lowercase(), "hellish rebuke", "2nd spell action should be hellish rebuke");
    assert_eq!(spell_actions[1].damage_roll.to_string(), "2d10 Fire", "Hellish rebuke damage roll should be 2d10 Fire");
    assert_eq!(spell_actions[2].name.to_lowercase(), "hellish rebuke", "3nd spell action should be hellish rebuke");
    assert_eq!(spell_actions[2].damage_roll.to_string(), "3d10 Fire", "Hellish rebuke damage roll should be 3d10 Fire");
    

    // pact slots should be 2 at level 10, and level 5
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 2, level: 5 }), "Pact slots at level 10 did not match expected value");

    // cast eldritch blast 3 times
    for _ in 0..3 {
        let eldritch_blast = spell_actions.first().expect("Couldn't get eldritch blast spell attack");
        baroopa.cast(eldritch_blast, None);
    }
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 2, level: 5 }), "Pact slots after casting eldritch blast 3 times did not match expected value");

    // Casting hellish rebuke at 5th level
    let hellish_rebuke = spell_actions.get(5).expect("Couldn't get hellish rebuke spell attack");
    assert_eq!(hellish_rebuke.damage_roll.to_string(), "6d10 Fire", "Hellish rebuke damage roll when cast at 5th level did not match expected value");
    baroopa.cast(hellish_rebuke, None);
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 1, level: 5 }), "Pact slots after casting hellish rebuke did not match expected value");

    // Casting blight at 5th level
    let blight = spell_actions.get(7).expect("Couldn't get blight spell attack");
    assert_eq!(blight.damage_roll.to_string(), "9d8 Necrotic", "Blight damage roll when cast at 5th level did not match expected value");
    baroopa.cast(blight, None);
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 0, level: 5 }), "Pact slots after casting blight did not match expected value");

    // short rest should restore pact slots
    baroopa.short_rest(0, None);
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 2, level: 5 }), "Pact slots after short rest did not match expected value");

    // long rest should restore pact slots
    baroopa.cast(hellish_rebuke, None);
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 1, level: 5 }), "Pact slots after casting hellish rebuke did not match expected value");
    baroopa.long_rest();
    assert_eq!(baroopa.available_pact_slots, Some(PactSlots {num: 2, level: 5 }), "Pact slots after long rest did not match expected value");
}
