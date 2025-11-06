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

pub fn value_name(v: &Value) -> &str {
    match v {
        Value::Object(_) => "Map",
        Value::Array(_) => "Array",
        Value::String(_) => "String",
        Value::Number(_) => "Number",
        Value::Bool(_) => "Bool",
        Value::Null => "Null"
    }
}

impl ValueExt for Value {
    fn as_string(&self, name: &str) -> Result<String, CharacterDataError> {
        self.as_str()
            .ok_or(CharacterDataError::mismatch(name, "String", value_name(self)))
            .map(|v| v.to_string())
    }

    fn get_str(&self, key: &str) -> Result<String, CharacterDataError> {
        Ok(self.get(key)
            .ok_or_else(|| CharacterDataError::not_found("String", key))?
            .as_str()
            .ok_or_else(|| CharacterDataError::mismatch(key, "String", value_name(self)))?
            .to_string())
    }

    fn get_usize(&self, key: &str) -> Result<usize, CharacterDataError> {
       Ok(self.get(key)
            .ok_or_else(|| CharacterDataError::not_found("Number", key))?
            .as_number()
            .ok_or_else(|| CharacterDataError::mismatch(key, "Number", value_name(self)))?
            .as_u64()
            .ok_or_else(|| CharacterDataError::mismatch(key, "unsigned integer", "signed int or float"))?
            .try_into()
            .expect("number too large"))
    }

    fn get_bool(&self, key: &str) -> Result<bool, CharacterDataError> {
        self.get(key)
            .ok_or_else(|| CharacterDataError::not_found("Bool", key))?
            .as_bool()
            .ok_or_else(|| CharacterDataError::mismatch(key, "Bool", value_name(self)))
    }


    fn get_map(&self, key: &str) -> Result<&Value, CharacterDataError> {
        let v = self.get(key)
            .ok_or_else(|| CharacterDataError::not_found("Map", key))?;
        if !v.is_object() {
            return Err(CharacterDataError::mismatch(key, "Map", value_name(self)))
        }
        Ok(v)
    }


    fn get_array(&self, key: &str) -> Result<&[Value], CharacterDataError> {
        Ok(self.get(key)
            .ok_or_else(|| CharacterDataError::not_found("Array", key))?
            .as_array()
            .ok_or_else(|| CharacterDataError::mismatch(key, "unsigned integer", "signed int or float"))?
            .as_ref())
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
            o => Err(CharacterDataError::mismatch("Description field", "String", value_name(o)))
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
pub fn choice<'a>(map_value: &'a Value) -> Result<NameCountMap<'a>, CharacterDataError> {
    let map = map_value.as_object()
        .ok_or_else(|| CharacterDataError::mismatch("choice", "Object", value_name(map_value)))?;

    let count = map_value.get_usize("choose")?;

    let description = map.get("desc")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let choice_arr = map.get("from")
        .ok_or_else(|| CharacterDataError::not_found("Object", "from"))?;

    let value_choices = process_bare_choice(choice_arr)?;

    Ok((description, count, value_choices))
}

fn process_bare_choice(choice_array: &Value) -> Result<PresentedOption<&Map<String, Value>>, CharacterDataError> {
    let choice_array = choice_array.as_object()
        .ok_or_else(|| CharacterDataError::mismatch("choice", "Object", value_name(choice_array)))?;

    // if we're at a base choice, return
    if let Some(Value::String(s)) = choice_array.get("option_type") {
        if s == "choice" {
            // getting the choice array and unwrapping the value
            let choice_val = choice_array.get("choice")
                .ok_or_else(|| CharacterDataError::not_found("Object", "choice object"))?;
            return Ok(choice(choice_val)?.2);
        } else if s == "multiple" {
            // TODO
            // i'd have to restructure a good bit to allow for multiple in a PresentedOption,
            // so for now it's just the first item. Later ill implement it as Base: Vec<T>
            let items_arr = match choice_array.get("items") {
                Some(Value::Array(a)) => a,
                Some(o) => return Err(CharacterDataError::mismatch("choice items", "Array", value_name(o))),
                None => return Err(CharacterDataError::not_found("Array", "choice items"))
            };
            if items_arr.is_empty() {
                return Err(CharacterDataError::not_found("Object items first entry", "object items"));
            };
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
        let assembled_choice  = a
            .iter()
            .map(process_bare_choice)
            .collect::<Result<Vec<_>,_>>()?
            .into_iter()
            .map(|v| v.as_base()
                .ok_or_else(|| CharacterDataError::mismatch("Choice option field", "One dimensional choice", "recursive choice")) 
                .map(|v| *v)
            )
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(PresentedOption::Choice(assembled_choice));
    };

    Err(CharacterDataError::not_found("Choice identifier", "option_type"))
}
