use memoizee::memoize;
use serde_json::Value;
use super::{get_page::get_raw_json, json_tools::{parse_string, ValueExt}, ValueError};
use crate::character::items::{Armor, ArmorCategory, DamageRoll, DamageType, Item, ItemType, Weapon, WeaponType};

pub async fn get_item(name: &str) -> Result<Item, ValueError> {
    let index = parse_string(name);
    get_item_raw(index).await
}

#[memoize]
async fn get_item_raw(index_name: String) -> Result<Item, ValueError> {
    let item_json = get_raw_json(format!("equipment/{index_name}")).await?;

    let name = item_json.get_str("name")?;
    let catagory = item_json.get_map("equipment_category")?.get_str("index")?;

    if name == "Shield" {
        return Ok(Item {
            name: "Shield".to_string(),
            description: None,
            item_type: ItemType::Shield,
            features: vec![],
        });
    }

    let item_type = match catagory.as_str() {
        "weapon" => ItemType::Weapon(weapon(&item_json)?),
        "armor" => ItemType::Armor(armor(&item_json)?),
        _ => ItemType::Misc,
    };


    let item = Item {
        name, 
        description: None,
        item_type,
        features: vec![]
    };
            
    Ok(item)

}

fn weapon(map: &Value) -> Result<Weapon, ValueError> {

    let damage_map = map.get_map("damage")?;

    let damage_type = damage_map.get_map("damage_type")?.get_str("index")?;

    let damage_type = DamageType::from_string(&damage_type)
        .map_err(|_| ValueError::ValueMismatch("damage type".to_string()))?;

    let damage = DamageRoll::from_str(&damage_map.get_str("damage_dice")?, damage_type)
        .ok_or_else(|| ValueError::ValueMismatch("damage roll".to_string()))?;

    let category_string = map.get_str("category_range")?;

    let weapon_type = match category_string.as_str() {
        "Simple Melee" => WeaponType::Simple,
        "Simple Ranged" => WeaponType::SimpleRanged,
        "Martial Melee" => WeaponType::Martial,
        "Martial Ranged" => WeaponType::MartialRanged,
        _ => return Err(ValueError::ValueMismatch("Weapon Type".to_string())),
    };

    let weapon = Weapon {
        damage,
        attack_roll_bonus: 0,
        weapon_type,
    };

    Ok(weapon)
}

fn armor(map: &Value) -> Result<Armor, ValueError> {

    let armor_class_map = map.get_map("armor_class")?;
    let ac = armor_class_map.get_usize("base")? as isize;

    let category = match map.get_str("armor_category")?.as_str() {
        "Light" => ArmorCategory::Light,
        "Medium" => ArmorCategory::Medium,
        "Heavy" => ArmorCategory::Heavy,
        _ => return Err(ValueError::ValueMismatch("Armor category".to_string())),
    };

    let strength_minimum = match map.get_usize("str_minimum")? {
        0 => None,
        other => Some(other),
    };

    let stealth_disadvantage = map.get_bool("stealth_disadvantage")?;

    let armor = Armor {
        ac,
        category,
        strength_minimum,
        stealth_disadvantage,
    };

    Ok(armor)
}



#[cfg(test)]
mod tests {

    use crate::character::items::{ArmorCategory, ItemType, WeaponType};

    use super::get_item;
    #[tokio::test]
    async fn shortsword_retrieval() {
        let v = get_item("shortsword").await.expect("Failed to get item");
        assert_eq!(v.name, "Shortsword", "Invalid field in item retrieval");

        let weapon = match v.item_type {
            ItemType::Weapon(w) => w,
            _ => panic!("Shortsword should be a weapon!"),
        };

        assert_eq!(weapon.weapon_type, WeaponType::Martial);
    }

    #[tokio::test]
    async fn studded_leather_retrieval() {
        let v = get_item("studded leather armor").await.expect("Failed to get studded leather");
        assert_eq!(v.name, "Studded Leather Armor");
        
        let armor = match v.item_type {
            ItemType::Armor(a) => a,
            _ => panic!("Studded leather armor should be armor!"),
        };

        assert_eq!(armor.ac, 12);
        assert_eq!(armor.category, ArmorCategory::Light);
    }

    #[tokio::test]
    async fn shield_retrieval() {
        let v = get_item("shield").await.expect("Failed to get shield");
        assert_eq!(v.name, "Shield");

        match v.item_type {
            ItemType::Shield => (),
            _ => panic!("Shield shoud have the shield type"),
        }
    }

}
