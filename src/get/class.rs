use std::collections::HashMap;
use serde_json::{Map, Value};
use crate::{character::{
    class::{Class, ItemCategory, Subclass}, 
    features::{Feature, PresentedOption}, 
    items::{Item, ItemType, WeaponType}, 
    spells::{SpellCasterType, SpellCastingPreperation, Spellcasting}, 
    stats::{EquipmentProficiencies, SkillType, StatType}
}, getter::DataProvider};

use crate::get::{
    feature::get_feature, 
    get_page::get_raw_json, 
    subclass::get_subclass,
    json_tools::{
        value_name, array_index_values, choice, parse_string, unwrap_number, ValueExt
    }, 
};
use crate::getter::CharacterDataError;


/// Get a class from the api
///
/// Note that this function takes a large amount of time, anywhere from 2 to 15 seconds. Try to run
/// it in the background when you can.
pub async fn get_class(getter: &impl DataProvider, class_name: &str) -> Result<Class, CharacterDataError> {
    let c = parse_string(class_name);
    let class_json= get_raw_json(format!("classes/{}", c))
        .await?;

    let levels_json = get_raw_json(format!("classes/{}/levels", c))
        .await?;

    json_to_class(getter, class_json, levels_json).await
}


async fn subclasses(map: &Value) -> Result<Vec<Subclass>, CharacterDataError> {
    let subclass_val_array = map.get_array("subclasses")?;
    
    let mut subclasses: Vec<Subclass> = Vec::with_capacity(subclass_val_array.len());
    for subclass_val in subclass_val_array.iter() {
        let subclass_index = subclass_val.get_str("index")?;
        let subclass = get_subclass(&subclass_index).await?;
        subclasses.push(subclass);
    }

    Ok(subclasses)
}

fn equipment_proficiencies_inner(proficiency_strings: Vec<String>) -> EquipmentProficiencies {
    let mut equipment = EquipmentProficiencies::default();

    let other = proficiency_strings.into_iter().filter(|v| {
        match v.as_ref() {
            "simple weapons" => {
                equipment.simple_weapons = true;
                false
            },
            "martial weapons" => {
                equipment.martial_weapons = true;
                false
            },
            "light armor" => {
                equipment.light_armor = true;
                false
            },
            "medium armor" => {
                equipment.medium_armor = true;
                false
            },
            "heavy armor" => {
                equipment.heavy_armor = true;
                false
            },
            "shields" => {
                equipment.shields = true;
                false
            }
            _ => true,
        }
    }).collect();

    equipment.other = other;

    equipment
}

fn equipment_proficiencies(json: &Value) -> Result<EquipmentProficiencies, CharacterDataError> {
    let proficiencies_json = json.get_array("proficiencies")?;
    let mut proficiency_strings_vec: Vec<String> = vec![];
    for proficiency_json in proficiencies_json.iter() {
        let name = proficiency_json.get_str("name")?.to_lowercase();
        if name.get(..=11) != Some("saving throw") {
            proficiency_strings_vec.push(name);
        }
    }

    Ok(equipment_proficiencies_inner(proficiency_strings_vec))
}

fn saves(json: &Value) -> Result<Vec<StatType>, CharacterDataError> {
    let saving_throws = json.get("saving_throws")
        .ok_or_else(|| CharacterDataError::not_found("Object", "character saving throws"))?;

    array_index_values(saving_throws, "name")
        .unwrap_or_default()
        .into_iter()
        .map(|s| StatType::from_shorthand(s.as_str()))
        .map(|s| s.ok_or_else(|| CharacterDataError::mismatch("saving throw", "vaild StatType string", "invalid StatType string")))
        .collect::<Result<Vec<_>, _>>()
}

fn proficiency_choices(map: &Value) -> Result<(usize, PresentedOption<SkillType>), CharacterDataError> {
    let proficiency_choice_array = map.get_array("proficiency_choices")?;

    let first_choice = proficiency_choice_array.first()
        .ok_or_else(|| CharacterDataError::mismatch("array", "array", "empty array"))?;

    // gets the choices in json values
    let (_, count, options) = choice(first_choice)?;

    // converts from json to skill types
    let proficiency_options  =  options.map(|val_map| {
        let val_obj = Value::Object(val_map.clone());
        let val = val_obj.get_map("item")?;

        let name = val.get_str("name")?;

        name.get(7..)
            .and_then(SkillType::from_name)
            .ok_or_else(|| CharacterDataError::mismatch("proficiency name", "Valid SkillType string", "Invalid SkillType string"))
    }).collect_result()?;

    Ok((count, proficiency_options))
}

async fn items(getter: &impl DataProvider, map: &Value) -> Result<Vec<PresentedOption<Vec<(ItemCategory, usize)>>>, CharacterDataError>  {
    let given_equipment = map.get_array("starting_equipment")?;

    // essentially a map without the async bs
    let mut equipment: Vec<PresentedOption<Vec<(ItemCategory, usize)>>> = Vec::with_capacity(given_equipment.len()+2);

    for equipment_value in given_equipment.iter() {
        if !equipment_value.is_object() {
            return Err(CharacterDataError::mismatch("equipment instance", "Object", value_name(equipment_value)))
        }

        let num = equipment_value.get_usize("quantity")?;

        let item = process_equipment(getter, &equipment_value["equipment"])
            .await?;

        equipment.push(PresentedOption::Base(vec![(ItemCategory::Item(item), num)]));
    }

    let equipment_options_arr = map.get_array("starting_equipment_options")?;

    for equipment_option in equipment_options_arr.iter() {
        let new_equipment = class_item_choice(getter, equipment_option).await?;
        equipment.push(new_equipment);
    }

    Ok(equipment)
}

async fn class_item_choice(getter: &impl DataProvider, equipment_option: &Value) -> Result<PresentedOption<Vec<(ItemCategory, usize)>>, CharacterDataError> {
    let (_, _, map_option) = choice(equipment_option)?;
    let v: PresentedOption<Result<Vec<(ItemCategory, usize)>, CharacterDataError>> = map_option.map_async(|m| async move {

        let count = m.get("count")
            .and_then(|v| v.as_number())
            .map(unwrap_number)
            .unwrap_or(1);

        // if its an equipment category rather than an item,
        if let Some(Value::String(s)) = m.get("option_set_type") {
            if s == "equipment_category" {
                let v = m.get("equipment_category")
                    .ok_or_else(|| CharacterDataError::not_found("Object", "equipment category"))?;
                let category = equipment_category(v)
                    .map_err(|v| v.prepend("equipment category "))?;
                // return the proper function
                return Ok(vec![(category, 1)])
            }
        }

        let equipment = m.get("of")
            .ok_or_else(|| CharacterDataError::not_found("Object", "equipment of"))?;

        let item = process_equipment(getter, equipment).await?;

        Ok(vec![(ItemCategory::Item(item), count)])
    }).await;

    v.collect_result()
}


fn equipment_category(map: &Value) -> Result<ItemCategory, CharacterDataError> {
    let equipment_name = map.get_str("name")?;

    match equipment_name.as_str() {
        "Simple Weapons" => return Ok(ItemCategory::Weapon(WeaponType::Simple)),
        "Martial Weapons" => return Ok(ItemCategory::Weapon(WeaponType::Martial)),
        _ => ()
    }

    let item = Item {
        name: equipment_name.clone(),
        description: None,
        item_type: ItemType::Misc,
        features: vec![],
    };

    Ok(ItemCategory::Item(item))
}

async fn process_equipment(getter: &impl DataProvider, val: &Value) -> Result<Item, CharacterDataError> {
    let index = val.get_str("index")?;
    getter.get_item(&index).await
}

async fn class_features(levels_arr: [&Value; 20])  -> Result<[Vec<PresentedOption<Feature>>; 20], CharacterDataError> {
    let mut levels_vec = Vec::with_capacity(20);

    for level in levels_arr.iter() {
        levels_vec.push(get_features_from_class_level(level).await?);
    }

    levels_vec
        .try_into()
        .map_err(|v: Vec<_>| CharacterDataError::mismatch("features per level vec", "array of size 20", &format!("array of size {}", v.len())))
}

async fn get_features_from_class_level(level: &Value) -> Result<Vec<PresentedOption<Feature>>, CharacterDataError> {

    let features_vals = level.get_array("features")?;

    let mut features_vec = Vec::with_capacity(features_vals.len());

    for f in features_vals {
        let feature_index = f.get_str("index")?;
        let feature = get_feature(&feature_index).await?;
        features_vec.push(PresentedOption::Base(feature));
    }

    Ok(features_vec)
}

fn spell_slots_from_map(json: &Value) -> Result<usize, CharacterDataError>{

    let slot_vals = json.as_object()
        .ok_or_else(|| CharacterDataError::mismatch("slots_vals", "Object", value_name(json)))?
        .values().map(|v|  v.as_number().and_then(|v| v.as_u64().map(|m| m as usize)))
        .collect::<Option<Vec<usize>>>()
        .ok_or_else(|| CharacterDataError::mismatch("Slots vals", "Usize applicable number", "Non-usize applicable value"))?;

    if slot_vals.is_empty() {
        return Err(CharacterDataError::mismatch("spell slot values", "filled spell slots", "empty spell slots"))
    }

    Ok(slot_vals[0])
}

fn preperation_type(name: &str) -> Option<SpellCastingPreperation> {
    use SpellCastingPreperation::{Prepared, Known};
    match name {
        "wizard" => Some(Prepared),
        "cleric" => Some(Prepared),
        "druid" => Some(Prepared),
        "paladin" => Some(Prepared),
        "bard" => Some(Known),
        "sorcerer" => Some(Known),
        "warlock" => Some(Known),
        "ranger" => Some(Known),
        _ => None,
    }
}

fn spellcasting_type(name: &str) -> Option<SpellCasterType> {
    match name {
        "wizard" => Some(SpellCasterType::Full),
        "cleric" => Some(SpellCasterType::Full),
        "druid" => Some(SpellCasterType::Full),
        "sorcerer" => Some(SpellCasterType::Full),
        "bard" => Some(SpellCasterType::Full),
        "paladin" => Some(SpellCasterType::Half),
        "ranger" => Some(SpellCasterType::Half),
        "artificer" => Some(SpellCasterType::Half),
        "warlock" => Some(SpellCasterType::Warlock),
        _ => None,
    }
}

fn spell_slots(levels_arr: [&Value; 20]) -> Result<[usize; 20], CharacterDataError> {
    let mut spell_slots_vec = Vec::with_capacity(20);

    for level in levels_arr {

        let spellcasting_map = level.get_map("spellcasting")?;

        let spell_slots = spell_slots_from_map(spellcasting_map)?;
        spell_slots_vec.push(spell_slots);
    }

    let cantrip_slots = spell_slots_vec.try_into()
        .map_err(|e: Vec<usize>| CharacterDataError::mismatch("cantrip slots array", "array of size 20", &format!("array of size {}", e.len())))?;
    

    Ok(cantrip_slots)
}

fn spellcasting_ability(val: &Value) -> Result<Option<StatType>, CharacterDataError> {
    if val.get("spellcasting").is_none() {
        return Ok(None)
    }

    let ability_score_string = val.get_map("spellcasting")?
        .get_map("spellcasting_ability")?
        .get_str("index")?;

    let ability_score = StatType::from_shorthand(&ability_score_string)
        .ok_or_else(|| CharacterDataError::mismatch("ability score type", "valid StatType string", "invalid StatType string"))?;

    Ok(Some(ability_score))
}

fn process_spell_list(spells: Value) -> Result<[Vec<String>; 10], CharacterDataError> {
    let spells_input_array = spells.get_array("results")?;
    let mut spells_stored_array: [Vec<String>; 10] =  Default::default();
    for spell_input in spells_input_array.iter() {
        let name = spell_input.get_str("index")?.clone();
        let level = spell_input.get_usize("level")?;
        spells_stored_array[level].push(name);
    }
    Ok(spells_stored_array)
}

fn class_specific_map_parse(key: &str, map: &Map<String, Value>) -> Result<String, CharacterDataError> {
    match key {
        "martial_arts" => {
            let count = map.get("dice_count")
                .ok_or_else(|| CharacterDataError::not_found("string", "martial arts dice count"))?;
            let value = map.get("dice_value")
                .ok_or_else(|| CharacterDataError::not_found("string", "martial arts dice value"))?;
            Ok(format!("{}d{}", count, value))
        }
        "sneak_attack" => {
            let count = map.get("dice_count")
                .ok_or_else(|| CharacterDataError::not_found("string", "sneak attack dice count"))?;
            let value = map.get("dice_value") 
                .ok_or_else(|| CharacterDataError::not_found("string", "sneak attack dice count"))?;
            Ok(format!("{}d{}", count, value))
        }
        _ => Err(CharacterDataError::mismatch(" Map value", "Valid map", &format!("Invalid map of the key name {}", key)))
    }
}

fn class_specific(levels: [&Value; 20]) -> Result<HashMap<String, [String; 20]>, CharacterDataError> {
    // for now we'll use vecs, we'll convert it to an array once we're done
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    let level_1 = levels[0]
        .get_map("class_specific")?;
    let level_1_obj = level_1
        .as_object()
        .ok_or_else(|| CharacterDataError::mismatch("level 1", "Object", value_name(level_1)))?;

    for key in level_1_obj.keys() {
        if key == "creating_spell_slots" {continue} // sorcerer useless field
        map.insert(key.replace("_", " "), vec![]);
    }



    for level in levels {
        let class_specific = level.get_map("class_specific")?;
        let class_specific_map = class_specific.as_object()
            .ok_or_else(|| CharacterDataError::mismatch("class specific field", "Object", value_name(class_specific)))?;
        for key in class_specific_map.keys() {
            if key == "creating_spell_slots" {continue};
            let other_val = class_specific_map.get(key)
                .ok_or_else(|| CharacterDataError::not_found("Any", "Class specific field key"))?;
            let other_as_string: String = match other_val {
                Value::Number(n) => n.as_f64().unwrap().to_string(),
                Value::Bool(b) => b.to_string(),
                Value::String(s) => s.clone(),
                Value::Object(o) => class_specific_map_parse(key, o)?,
                v => return Err(CharacterDataError::mismatch("Class specific value", "Value that can be parsed into a string", value_name(v))),
            };

            map.get_mut(&key.replace("_", " "))
                .ok_or_else(|| CharacterDataError::not_found("Vec of class specific values",&format!("class specific field of key {}", key)))?
                .push(other_as_string);
        }
    }
            
    let mapped: HashMap<String, [String; 20]> = map
        .into_iter()
        .filter_map(|(k, v)| {
            if v.len() == 20 {
                let arr: [String; 20] = v.try_into().ok()?;
                Some((k, arr))
            } else {
                None // Skip entries that aren’t exactly 9 long
            }
        })
        .collect();

    Ok(mapped)
}

async fn process_spellcasting(json: &Value, levels_arr: [&Value; 20]) -> Result<Option<Spellcasting>, CharacterDataError> {
    let name = json.get_str("index")?;

    let spells = get_raw_json(format!("classes/{}/spells", name)).await?;

    let casting_ability = spellcasting_ability(json)?;
    let caster_type_option: Option<SpellCasterType> = spellcasting_type(name.as_ref());

    casting_ability.zip(caster_type_option)
        .map(|(spellcasting_ability, spellcaster_type)| {
            let spell_list = process_spell_list(spells)?;
            // This just returns the cantrips, since spell slots are handled elsewhere
            let cantrips_per_level = spell_slots(levels_arr)?;
            let preperation_type = preperation_type(name.as_ref())
                .ok_or_else(|| CharacterDataError::mismatch("spellcaster preperation type", "name within bounds to be parsed for preperation", "unrecognized class name"))?;

            Ok(Spellcasting {
                spellcasting_ability,
                cantrips_per_level,
                spell_list,
                spellcaster_type,
                preperation_type,
            })
        })
        .transpose()
}

fn multiclassing_prerequisites(name: &str) -> (HashMap<StatType, usize>, bool) {
    let mut prerequisites_map: HashMap<StatType, usize> = HashMap::new();
    let mut or_flag = false;


    // hardcoded, since the api implementation is a bit compex
    
    // match the class name, and insert the proper prerequisites into the prerequisites map
    match name {
        "barbarian" => { prerequisites_map.insert(StatType::Strength, 13); },
        "bard" => { prerequisites_map.insert(StatType::Charisma, 13); },
        "cleric" => { prerequisites_map.insert(StatType::Wisdom, 13); },
        "druid" => { prerequisites_map.insert(StatType::Wisdom, 13); },
        "fighter" => { prerequisites_map.insert(StatType::Strength, 13); prerequisites_map.insert(StatType::Dexterity, 13); or_flag = true; },
        "monk" => { prerequisites_map.insert(StatType::Dexterity, 13); prerequisites_map.insert(StatType::Wisdom, 13); },
        "paladin" => { prerequisites_map.insert(StatType::Strength, 13); prerequisites_map.insert(StatType::Charisma, 13); },
        "ranger" => { prerequisites_map.insert(StatType::Dexterity, 13); prerequisites_map.insert(StatType::Wisdom, 13); },
        "rogue" => { prerequisites_map.insert(StatType::Dexterity, 13); },
        "sorcerer" => { prerequisites_map.insert(StatType::Charisma, 13); },
        "warlock" => { prerequisites_map.insert(StatType::Charisma, 13); },
        "wizard" => { prerequisites_map.insert(StatType::Intelligence, 13); },
        _ => (),
    }

    (prerequisites_map, or_flag)
}

fn multiclassing_proficiencies(json: &Value) -> Result<EquipmentProficiencies, CharacterDataError> {
    let multiclassing_map = json.get_map("multi_classing")?;
    let proficiency_strings = multiclassing_map.get_array("proficiencies")?
        .iter()
        .map(|v| v.get_str("name"))
        .collect::<Result<Vec<String>, CharacterDataError>>()?;

    Ok(equipment_proficiencies_inner(proficiency_strings))
}

async fn json_to_class(getter: &impl DataProvider, json: Value, levels: Value) -> Result<Class, CharacterDataError> {

    let name: String = json.get_str("index")
        .map_err(|v| v.prepend("class name "))?;

    let hit_die: usize = json.get_usize("hit_die")?;
    
    let subclasses: Vec<Subclass> = subclasses(&json).await
        .map_err(|v| v.prepend("Subclass "))?;

    let saving_throw_proficiencies: Vec<StatType> = saves(&json).unwrap_or_default();
    let equipment_proficiencies = equipment_proficiencies(&json)?;
    let skill_proficiency_choices: (usize, PresentedOption<SkillType>) = proficiency_choices(&json)
        .map_err(|v| v.prepend("Skill choices "))?;
    let beginning_items = items(getter, &json).await
        .map_err(|v| v.prepend("items "))?;

    let levels_arr: [&Value; 20]  = levels.as_array()
        .ok_or_else(|| CharacterDataError::mismatch("levels json", "array", value_name(&levels)))?
        .iter().collect::<Vec<_>>()
        .try_into()
        .map_err(|v: Vec<&Value>| CharacterDataError::mismatch("levels json", "array of size 20", &format!("array of size {}", v.len())))?;
    
    let features = class_features(levels_arr).await?;

    let class_specific_leveled = class_specific(levels_arr)
        .map_err(|v| v.prepend("Class specific values"))?;
    
    let spellcasting = process_spellcasting(&json, levels_arr).await?;

    let (multiclassing_prerequisites, multiclassing_prerequisites_or) = multiclassing_prerequisites(&name);
    let multiclassing_proficiency_gain = multiclassing_proficiencies(&json)?;


    let class = Class {
        name,
        subclasses,
        features,
        beginning_items,
        saving_throw_proficiencies,
        equipment_proficiencies,
        hit_die,
        class_specific_leveled,
        skill_proficiency_choices,
        spellcasting,
        multiclassing_prerequisites,
        multiclassing_prerequisites_or,
        multiclassing_proficiency_gain,
    };

    Ok(class)
}
