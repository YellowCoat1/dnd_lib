//! shared tools for handling incoming json from the api.
use serde_json::{Value, Map, Number};
use crate::getter::CharacterDataError;

use crate::character::features::PresentedOption;
pub trait ValueExt {
    fn as_string(&self, name: &str) -> Result<String, CharacterDataError>;
    fn get_str(&self, key: &str) -> Result<String, CharacterDataError>;
    fn get_usize(&self, key: &str) -> Result<usize, CharacterDataError>;
    fn get_bool(&self, key: &str) -> Result<bool, CharacterDataError>;
    fn get_map(&self, key: &str) -> Result<&Value, CharacterDataError>;
    fn get_array(&self, key: &str) -> Result<&[Value], CharacterDataError>;
}

impl ValueExt for Value {
    fn as_string(&self, name: &str) -> Result<String, CharacterDataError> {
        self.as_str()
            .ok_or(CharacterDataError::ValueMismatch(String::from(name)))
            .map(|v| v.to_string())
    }

    fn get_str(&self, key: &str) -> Result<String, CharacterDataError> {
        self.get(key)
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .ok_or_else(|| CharacterDataError::ValueMismatch(key.to_string()))
    }

    fn get_usize(&self, key: &str) -> Result<usize, CharacterDataError> {
        self.get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v.try_into().unwrap())
            .ok_or_else(|| CharacterDataError::ValueMismatch(key.to_string()))
    }

    fn get_bool(&self, key: &str) -> Result<bool, CharacterDataError> {
        self.get(key)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| CharacterDataError::ValueMismatch(key.to_string()))
    }


    fn get_map(&self, key: &str) -> Result<&Value, CharacterDataError> {
        self.get(key)
            .and_then(|v| if v.is_object() { Some(v) } else {None})
            .ok_or_else(|| CharacterDataError::ValueMismatch(key.to_string()))

    }

    fn get_array(&self, key: &str) -> Result<&[Value], CharacterDataError> {
        self.get(key)
            .and_then(|v| v.as_array())
            .ok_or_else(|| CharacterDataError::ValueMismatch(key.to_string()))
            .map(|v| v.as_slice())
    }
}


// parses a string to be used as an index.
pub fn parse_string(s: &str) -> String {
    s.to_lowercase().replace(" ", "-")
}

pub fn string_array(arr: &[Value]) -> Result<Vec<String>, CharacterDataError> {
    arr.iter()
        .map(|v| match v {
            Value::String(s) => Ok(s.to_string()),
            _ => Err(CharacterDataError::ValueMismatch(String::from("description field"))),
        }).collect::<Result<Vec<String>, CharacterDataError>>()
}

pub fn object_index_value<'a>(object: &'a Value, index_name: &str) -> Result<&'a String, ()> {
    match &object[index_name] {
        Value::String(s) => Ok(s),
        _ => Err(()),
    }
}

// if there is a json value that is an array, grab a string value from that array from a string key.
pub fn array_index_values<'a>(array: &'a Value, index_name: &str) -> Result<Vec<&'a String>, ()> {

    let obj_vec = match array {
        Value::Array(a) => a,
        _ => return Err(()),
    };

    obj_vec.iter()
        .map(|v| object_index_value(v, index_name))
        .collect()
}

pub fn unwrap_number(num: &Number) -> usize {
    num.as_f64().unwrap() as usize
}

// description, count, value_choices
type NameCountMap<'a> = (String, usize, PresentedOption<&'a Map<String, Value>>);
pub fn choice<'a>(map_value: &'a Value) -> Result<NameCountMap<'a>, ()> {
    let map = match map_value {
        Value::Object(o) => o,
        _ => return Err(()),
    };

    let count = match map.get("choose") {
        Some(Value::Number(n)) => n.as_f64().unwrap() as usize,
        _ => return Err(()),
    };

    let description = match map.get("desc") {
        Some(Value::String(s)) => s.clone(),
        _ => String::from(""),
    };

    let choice_arr = match map.get("from") {
        Some(a) => a,
        _ => return Err(()),
    };

    let value_choices = process_bare_choice(choice_arr)?;

    Ok((description, count, value_choices))
}

fn process_bare_choice(choice_array: &Value) -> Result<PresentedOption<&Map<String, Value>>, ()> {
    let choice_array = match choice_array {
        Value::Object(o) => o,
        _ => panic!("A"),
        //_ => return Err(()),
    };
    
    // if we're at a base choice, return
    if let Some(Value::String(s)) = choice_array.get("option_type") {
        if s == "choice" {
            // getting the choice array and unwrapping the value
            let choice_val = match choice_array.get("choice") {
                Some(v) => v, 
                _ => return Err(())
            };
            return Ok(choice(choice_val)?.2);
        } else if s == "multiple" {
            // TODO
            // i'd have to restructure a good bit to allow for multiple in a PresentedOption,
            // so for now it's just the first item. Later ill implement it as Base: Vec<T>
            let items_arr = match choice_array.get("items") {
                Some(Value::Array(a)) => a,
                _ => return Err(())
            };
            if items_arr.is_empty() {return Err(())};
            return process_bare_choice(&items_arr[0]);
        }
        return Ok(PresentedOption::Base(choice_array));
    };

    let opt_type = match choice_array.get("option_set_type") {
        Some(Value::String(s)) => s.as_str(),
        _ => return Ok(PresentedOption::Base(choice_array)),
    };

    if opt_type != "options_array" {
        return Ok(PresentedOption::Base(choice_array));
    }

    if let Some(Value::Array(a)) = choice_array.get("options") {
        let assembled_choice: Vec<PresentedOption<&Map<String, Value>>> = a
            .iter()
            .map(process_bare_choice)
            .collect::<Result< Vec<PresentedOption<&Map<String, Value>>>, ()>>()?;
        return Ok(PresentedOption::Choice(assembled_choice));
    };

    Err(())
}
