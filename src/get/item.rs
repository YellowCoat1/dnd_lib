use super::Dnd5eapiError;
use super::{
    get_page::get_raw_json,
    json_tools::{parse_string, ValueExt},
};
use crate::rules2014::items::{
    Armor, ArmorCategory, DamageRoll, DamageType, Item, ItemType, Weapon, WeaponProperties,
    WeaponType,
};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum ItemError {
    #[error("err {0} with item {1}")]
    ParseError(String, String),
}

impl ItemError {
    fn parse_error(msg: &str, name: &str) -> ItemError {
        ItemError::ParseError(msg.to_string(), name.to_string())
    }
}
pub async fn get_item(name: &str) -> Result<Item, Dnd5eapiError> {
    let index = parse_string(name);
    get_item_raw(index).await
}

async fn get_item_raw(index_name: String) -> Result<Item, Dnd5eapiError> {
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
        "weapon" => ItemType::Weapon(weapon(&name, &item_json)?),
        "armor" => ItemType::Armor(armor(&name, &item_json)?),
        _ => ItemType::Misc,
    };

    let item = Item {
        name,
        description: None,
        item_type,
        features: vec![],
    };

    Ok(item)
}

fn weapon(name: &str, map: &Value) -> Result<Weapon, Dnd5eapiError> {
    let damage_map = map.get_map("damage")?;

    let damage_type = damage_map.get_map("damage_type")?.get_str("index")?;

    let damage_type = damage_type
        .parse()
        .map_err(|_| ItemError::parse_error("weapon damage type", name))?;

    let damage = DamageRoll::from_str(&damage_map.get_str("damage_dice")?, damage_type)
        .ok_or(ItemError::parse_error("weapon damage roll", name))?;

    let category_string = map.get_str("category_range")?;

    let weapon_type = match category_string.as_str() {
        "Simple Melee" => WeaponType::Simple,
        "Simple Ranged" => WeaponType::SimpleRanged,
        "Martial Melee" => WeaponType::Martial,
        "Martial Ranged" => WeaponType::MartialRanged,
        _ => {
            return Err(Dnd5eapiError::mismatch(
                "weapon type",
                "weapon string",
                "irregular string",
            ))
        }
    };

    let properties = properties(name, map, damage.damage_type)?;

    let weapon = Weapon {
        damage,
        attack_roll_bonus: 0,
        properties,
        weapon_type,
    };

    Ok(weapon)
}

fn properties(
    name: &str,
    map: &Value,
    damage_type: DamageType,
) -> Result<WeaponProperties, Dnd5eapiError> {
    let arr = map.get_array("properties")?;
    let two_handed_damage = map.get_map("two_handed_damage").ok();
    let mut properties = WeaponProperties::default();
    for v in arr.iter() {
        let index = v.get_str("index")?;
        match index.as_str() {
            "ammunition" => properties.ammunition = true,
            "finesse" => properties.finesse = true,
            "heavy" => properties.heavy = true,
            "light" => properties.light = true,
            "loading" => properties.loading = true,
            "monk" => properties.monk = true,
            "reach" => properties.reach = true,
            "special" => properties.special = true,
            "thrown" => properties.thrown = true,
            "two_handed" => properties.two_handed = true,
            "versatile" => {
                let damage_val =
                    two_handed_damage.ok_or(ItemError::parse_error("two handed damage", name))?;
                let damage = DamageRoll::from_str(&damage_val.get_str("damage_dice")?, damage_type)
                    .ok_or(ItemError::parse_error("two handed damage roll", name))?;
                properties.versatile = Some(damage);
            }
            _ => (),
        }
    }
    Ok(properties)
}
fn armor(name: &str, map: &Value) -> Result<Armor, Dnd5eapiError> {
    let armor_class_map = map.get_map("armor_class")?;
    let ac = armor_class_map.get_usize("base")? as isize;

    let category = match map.get_str("armor_category")?.as_str() {
        "Light" => ArmorCategory::Light,
        "Medium" => ArmorCategory::Medium,
        "Heavy" => ArmorCategory::Heavy,
        _ => return Err(ItemError::parse_error("armor category", name).into()),
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

    use crate::rules2014::items::{ArmorCategory, DamageRoll, DamageType, ItemType, WeaponType};

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
        let v = get_item("studded leather armor")
            .await
            .expect("Failed to get studded leather");
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

    #[tokio::test]
    async fn versatile() {
        let v = get_item("longsword")
            .await
            .expect("failed to get longsword");

        let weapon = match v.item_type {
            ItemType::Weapon(w) => w,
            _ => panic!("longsword is not a weapon"),
        };

        let versatile_damage = weapon
            .properties
            .versatile
            .expect("longsword should be versatile");

        assert_eq!(
            versatile_damage,
            DamageRoll::from_str("1d10", DamageType::Slashing).unwrap()
        )
    }
}
