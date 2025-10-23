use std::collections::HashMap;
use serde_json::{Value, Map};
use crate::{character::items::WeaponType, get::{
    feature::get_feature, get_page::get_raw_json, item::get_item, json_tools::{
        array_index_values, choice, parse_string, unwrap_number, ValueError, ValueExt
    }, subclass::get_subclass
}};
use crate::character::{
    stats::{StatType, SkillType},
    items::{Item, ItemType},
    features::{Feature, PresentedOption},
    spells::{Spellcasting, SpellSlots},
    class::{Class, Subclass, ItemCategory},
};

/// Get a class from the api
///
/// Note that this function takes a large amount of time, anywhere from 2 to 15 seconds. Try to run
/// it in the background when you can.
pub async fn get_class(class_name: &str) -> Result<Class, ValueError> {
    let c = parse_string(class_name);
    let class_json= get_raw_json(format!("classes/{}", c))
        .await?;

    let levels_json = get_raw_json(format!("classes/{}/levels", c))
        .await?;

    // spell list may or may not have a result. It only matters if we're parsing a spellcaster, so
    // we'll unwrap it when we know we're going to use it.
    let spell_list = get_raw_json(format!("classes/{}/spells", c)).await;

    json_to_class(class_json, levels_json, spell_list).await
}


async fn subclasses(map: &Value) -> Result<Vec<Subclass>, ValueError> {
    let subclass_val_array = map.get_array("subclasses")?;
    
    let mut subclasses: Vec<Subclass> = Vec::with_capacity(subclass_val_array.len());
    for subclass_val in subclass_val_array.iter() {
        let subclass_index = subclass_val.get_str("index")?;
        let subclass = get_subclass(&subclass_index).await?;
        subclasses.push(subclass);
    }

    Ok(subclasses)
}

fn saves(json: &Value) -> Vec<StatType> {
    let saving_throws_option = json.get("saving_throws");
    let saving_throws = match saving_throws_option {
        Some(v) => v,
        _ => return vec![], 
    };
    array_index_values(saving_throws, "name")
        .unwrap_or(vec![])
        .into_iter()
        .map(|s| StatType::from_shorthand(s.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or(vec![])
}

fn proficiency_choices(map: &Value) -> Result<(usize, PresentedOption<SkillType>), ValueError> {
    let proficiency_choice_array = map.get_array("proficiency_choices")?;

    let first_choice = proficiency_choice_array.get(0)
        .ok_or_else(|| ValueError::ValueMismatch("First proficiency choice".to_string()))?;

    let proficiency_choices_err = ValueError::ValueMismatch("Proficiency choices".to_string());


    // gets the choices in json values
    let (_, count, options) = choice(first_choice)
        .map_err(|_| proficiency_choices_err.clone())?;

    // converts from json to skill types
    let proficiency_options  =  options.map(|val| {
        let val = val.get("item")
            .map(|v| v.as_object())
            .flatten()
            .ok_or_else(|| proficiency_choices_err.clone().prepend("item for"))?;

        let name = val.get("name")
            .map(|v| v.as_str())
            .flatten()
            .ok_or_else(|| proficiency_choices_err.clone().prepend("name for "))?;

        let proficiency_name = name.get(7..)
            .ok_or_else(|| proficiency_choices_err.clone().prepend("name for "))?;

        SkillType::from_name(proficiency_name)
            .map_err(|_| proficiency_choices_err.clone().prepend("skill type for "))
    }).collect_result()?;

    return Ok((count, proficiency_options))
}

async fn items(map: &Value) -> Result<Vec<PresentedOption<Vec<(ItemCategory, usize)>>>, ValueError>  {
    let given_equipment = map.get_array("starting_equipment")?;

    // essentially a map without the async bs
    let mut equipment: Vec<PresentedOption<Vec<(ItemCategory, usize)>>> = Vec::with_capacity(given_equipment.len()+2);

    for equipment_value in given_equipment.iter() {
        if !equipment_value.is_object() {
            return Err(ValueError::ValueMismatch("equipment object".to_string()));
        }

        let num = equipment_value.get_usize("quantity")?;

        let item = process_equipment(&equipment_value["equipment"])
            .await?;

        equipment.push(PresentedOption::Base(vec![(ItemCategory::Item(item), num)]));
    }

    let equipment_options_arr = map.get_array("starting_equipment_options")?;

    for equipment_option in equipment_options_arr.iter() {
        let new_equipment = class_item_choice(equipment_option).await?;
        equipment.push(new_equipment);
    }

    Ok(equipment)
}

async fn class_item_choice(equipment_option: &Value) -> Result<PresentedOption<Vec<(ItemCategory, usize)>>, ValueError> {
    let (_, _, map_option) = choice(equipment_option)
        .map_err(|_| ValueError::ValueMismatch("choice".to_string()))?;
    let v: PresentedOption<Result<Vec<(ItemCategory, usize)>, ValueError>> = map_option.map_async(|m| async move {

        let count = m.get("count")
            .map(|v| v.as_number())
            .flatten()
            .map(|v| unwrap_number(v))
            .unwrap_or(1);

        // if its an equipment category rather than an item,
        if let Some(Value::String(s)) = m.get("option_set_type") {
            if s == "equipment_category" {
                let v = m.get("equipment_category")
                    .ok_or_else(|| ValueError::ValueMismatch("equipment get".to_string()))?;
                let category = equipment_category(v)
                    .map_err(|v| v.prepend("equipment category "))?;
                // return the proper function
                return Ok(vec![(category, 1)])
            }
        }

        let equipment = m.get("of")
            .ok_or_else(|| ValueError::ValueMismatch("equipment".to_string()))?;

        let item = process_equipment(equipment).await?;

        Ok(vec![(ItemCategory::Item(item), count)])
    }).await;

    Ok(v.collect_result()?)
}


fn equipment_category(map: &Value) -> Result<ItemCategory, ValueError> {
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

async fn process_equipment(val: &Value) -> Result<Item, ValueError> {
    let index = val.get_str("index")?;
    get_item(&index).await
}

async fn class_features(levels_arr: &Vec<Value>)  -> Result<Vec<Vec<PresentedOption<Feature>>>, ()> {
    let mut levels_vec = Vec::with_capacity(20);

    for level in levels_arr.iter() {
        levels_vec.push(get_features_from_class_level(level).await?)
    }

    Ok(levels_vec)
}

async fn get_features_from_class_level(level: &Value) -> Result<Vec<PresentedOption<Feature>>, ()> {
    let level_map = match level {
        Value::Object(o) => o,
        _ => return Err(())
    };

    let features_vals = match level_map.get("features") {
        Some(Value::Array(a)) => a,
        _ => return Err(()),
    };

    let mut features_vec = Vec::with_capacity(features_vals.len());

    for f in features_vals {
        let feature_map = match f {
            Value::Object(o) => o,
            _ => return Err(()),
        };

        let feature_index = match feature_map.get("index") {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(()),
        };

        let feature = get_feature(&feature_index).await.map_err(|_| ())?;
        features_vec.push(PresentedOption::Base(feature));
    }

    Ok(features_vec)
}

fn spell_slots_from_map(map: &Map<String, Value>) -> Result<SpellSlots, ()>{
    let slot_vals = map.values().map(|v| match v {
        Value::Number(n) => Ok(unwrap_number(n)),
        _ => Err(())
    }).collect::<Result<Vec<usize>, ()>>()?;

    if slot_vals.len() != 10 {
        return Err(());
    }

    let spell_slots = SpellSlots(slot_vals[0], slot_vals[1], slot_vals[2], slot_vals[3], slot_vals[4], slot_vals[5], slot_vals[6],
        slot_vals[7], slot_vals[8], slot_vals[9]);


    Ok(spell_slots)
}

fn spell_slots(levels_arr: &Vec<Value>) -> Result<Option<Vec<SpellSlots>>, ()> {
    let mut spell_slots_vec = Vec::with_capacity(20);

    for level_val in levels_arr {
        let level_map = match level_val {
            Value::Object(o) => o,
            _ => return Err(()),
        };

        let spellcasting_map = match level_map.get("spellcasting") {
            Some(Value::Object(o)) => o,
            _ => return Ok(None),
        };

        let spell_slots = spell_slots_from_map(spellcasting_map)?;
        spell_slots_vec.push(spell_slots);
    }

    Ok(Some(spell_slots_vec))
}

fn spellcasting_ability(val: &Value) -> Result<Option<StatType>, ()> {
    if val.get("spellcasting").is_none() {
        return Ok(None)
    }

    let ability_score_string = val.get_map("spellcasting").map_err(|_| ())?
        .get_map("spellcasting_ability").map_err(|_| ())?
        .get_str("index").map_err(|_| ())?;

    let ability_score = StatType::from_shorthand(&ability_score_string)?;

    Ok(Some(ability_score))
}

fn spell_list(spells: Value) -> Result<[Vec<String>; 10], ValueError> {
    let spells_input_array = spells.get_array("results")?;
    let mut spells_stored_array: [Vec<String>; 10] =  Default::default();
    for spell_input in spells_input_array.iter() {
        let name = spell_input.get_str("index")?.clone();
        let level = spell_input.get_usize("level")?;
        spells_stored_array[level].push(name);
    }
    Ok(spells_stored_array)
}

fn class_specific(levels: &Vec<Value>) -> Result<HashMap<String, [String; 20]>, ValueError> {
    // for now we'll use vecs, we'll convert it to an array once we're done
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    let class_specific_err = ValueError::ValueMismatch("class specific level features".to_string());

    let level_1 = levels.get(0)
        .ok_or_else(||ValueError::ValueMismatch("levels".to_string()))?;
    let level_1_class_specific = level_1.get_map("class_specific")?;
    let level_1_class_specific_map = match level_1_class_specific {
        Value::Object(o) => o,
        _ => return Err(class_specific_err.clone().append(" map"))
    };
    
    for key in level_1_class_specific_map.keys().cloned() {
        map.insert(key.replace("_", " "), vec![]);
    }



    for level in levels {
        let class_specific_map = level.get_map("class_specific")?.as_object()
            .ok_or_else(|| class_specific_err.clone().append(" object"))?;
        for key in class_specific_map.keys() {
            let other_val = class_specific_map.get(key).ok_or_else(|| class_specific_err.clone().append(" unfound key"))?;
            let other_as_string: String = match other_val {
                Value::Number(n) => n.as_u64().ok_or_else(|| class_specific_err.clone().append(" number???"))?.to_string(),
                Value::String(s) => s.clone(),
                Value::Object(o) => {
                    if key == "martial_arts" {
                        let count = o.get("dice_count")
                            .ok_or_else(|| class_specific_err.clone().append(" martial arts dice count"))?;
                        let value = o.get("dice_count")
                            .ok_or_else(|| class_specific_err.clone().append(" martial arts dice value"))?;
                        format!("{}d{}", count, value)
                    } else {
                        return Err(class_specific_err.clone().append(" unrecognized val obj"));
                    }
                },
                _ => return Err(class_specific_err.clone().append(" unrecognized val")),
            };
            map.get_mut(&key.replace("_", " ")).ok_or_else(|| class_specific_err.clone().append(" unrecognized key"))?.push(other_as_string);
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

async fn json_to_class(json: Value, levels: Value, spells: Result<Value, reqwest::Error>) -> Result<Class, ValueError> {

    let name: String = json.get_str("index")
        .map_err(|_| ValueError::ValueMismatch("Couldn't get name".to_string()))?;

    let hit_die: usize = json.get_usize("hit_die")?;
    
    let subclasses: Vec<Subclass> = subclasses(&json).await
        .map_err(|v| v.prepend("Subclass "))?;

    let saving_throw_proficiencies: Vec<StatType> = saves(&json);
    let skill_proficiency_choices: (usize, PresentedOption<SkillType>) = proficiency_choices(&json)?;
    let beginning_items = items(&json).await
        .map_err(|v| v.prepend("items "))?;

    let levels_arr = levels.as_array()
        .ok_or_else(|| ValueError::ValueMismatch("levels array".to_string()))?;
    
    let features = match class_features(&levels_arr).await {
        Ok(v) => v,
        _ => vec![],
    };

    let class_specific_leveled = class_specific(&levels_arr)?;


    let spellcasting_ability = spellcasting_ability(&json).map_err(|_| ValueError::ValueMismatch("spellcasting ability".to_string()))?;
    let spell_slots_per_level = spell_slots(&levels_arr).ok().flatten();
    let spellcasting = if let (Some(ability), Some(slots)) = (spellcasting_ability, spell_slots_per_level) {
        let spell_list = spell_list(spells?)?;
        Some(Spellcasting {
            spellcasting_ability: ability,
            spell_slots_per_level: slots,
            spell_list,
        })
    } else {
        None
    };

    let class = Class {
        name,
        subclasses,
        features,
        beginning_items,
        saving_throw_proficiencies,
        hit_die,
        skill_proficiency_choices,
        spellcasting,
        class_specific_leveled,
    };

    Ok(class)
}
