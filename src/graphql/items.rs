use graphql_client::{GraphQLQuery, Response};
use std::{collections::HashMap, str::FromStr};

use thiserror::Error;

use crate::{
    graphql::items::equipment_query::{EquipmentQueryEquipments, ResponseData},
    rules2014::items::{
        Armor, DamageRoll, DamageType, Item, ItemType, Weapon, WeaponProperties, WeaponType,
    },
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_query/schema.json"
    query_path = "graphql_query/query.graphql"
    response_derives = "Debug",
)]
struct EquipmentQuery;

#[derive(Debug, Error)]
#[error("{name}: {source}", name = name.as_deref().unwrap_or(""))]
pub struct ItemError {
    pub source: ItemErrorVariant,
    pub name: Option<String>,
}

impl ItemError {
    pub fn new(variant: ItemErrorVariant) -> ItemError {
        ItemError {
            source: variant,
            name: None,
        }
    }
    pub fn named(variant: ItemErrorVariant, name: String) -> ItemError {
        ItemError {
            source: variant,
            name: Some(name),
        }
    }
}

#[derive(Debug, Error)]
pub enum ItemErrorVariant {
    #[error("Failed to obtain item database!")]
    DatabaseLockError,
    #[error("Item does not exist.")]
    DoesNotExist,
    #[error("Failed to contact API.")]
    ConnectionError,
    #[error("Failed to parse {0}")]
    ParseError(String),
}

pub async fn new_database() -> Result<HashMap<String, Item>, ItemError> {
    let response = request_equipment()
        .await
        .ok_or(ItemError::new(ItemErrorVariant::ConnectionError))?;
    response
        .equipments
        .iter()
        .map(process_equipment)
        .collect::<Result<HashMap<String, Item>, ItemError>>()
}

async fn request_equipment() -> Option<ResponseData> {
    //get data
    let vars = equipment_query::Variables;
    let request_body = EquipmentQuery::build_query(vars);

    let client = reqwest::Client::new();
    let res = client
        .post("https://www.dnd5eapi.co/graphql")
        .json(&request_body)
        .send()
        .await
        .ok()?;
    let response_body: Response<equipment_query::ResponseData> = res.json().await.ok()?;
    response_body.data
}

fn process_equipment(
    api_equipment: &EquipmentQueryEquipments,
) -> Result<(String, Item), ItemError> {
    use EquipmentQueryEquipments::*;
    let item = match api_equipment {
        Armor(armor) => process_armor(armor)?,
        Weapon(weapon) => process_weapon(weapon)?,
        Tool(tool) => generic_item(&tool.name, &tool.desc),
        Gear(gear) => generic_item(&gear.name, &gear.desc),
        Pack(pack) => generic_item(&pack.name, &pack.desc),
        Ammunition(ammo) => generic_item(&ammo.name, &ammo.desc),
        Vehicle(vehicle) => generic_item(&vehicle.name, &vehicle.desc),
    };

    Ok((item.name.to_lowercase(), item))
}

fn process_armor(
    armor: &equipment_query::EquipmentQueryEquipmentsOnArmor,
) -> Result<Item, ItemError> {
    use crate::rules2014::items::ArmorCategory;
    let name = armor.name.clone();

    let description = match armor.desc.as_ref().unwrap_or(&vec![]).join(" ").trim() {
        "" => None,
        o => Some(o.to_string()),
    };

    if &armor.armor_category.to_lowercase() == "shield" {
        return Ok(Item {
            name,
            description,
            item_type: ItemType::Shield,
            features: vec![],
        });
    }

    let ac: isize = armor.armor_class.base.try_into().map_err(|_| {
        ItemError::named(
            ItemErrorVariant::ParseError("Armor Class".to_string()),
            name.clone(),
        )
    })?;
    let category = ArmorCategory::from_str(&armor.armor_category).map_err(|_| {
        ItemError::named(
            ItemErrorVariant::ParseError("Armor Category".to_string()),
            name.clone(),
        )
    })?;
    let stealth_disadvantage = armor.stealth_disadvantage.unwrap_or(false);
    let strength_minimum: Option<usize> = armor
        .str_minimum
        .map(|v| v.try_into())
        .transpose()
        .map_err(|_| {
            ItemError::named(
                ItemErrorVariant::ParseError("Strength Minimum".to_string()),
                name.clone(),
            )
        })?;

    let item_type = ItemType::Armor(Armor {
        ac,
        category,
        stealth_disadvantage,
        strength_minimum,
    });

    Ok(Item {
        name,
        description,
        item_type,
        features: vec![],
    })
}

fn process_weapon(
    weapon: &equipment_query::EquipmentQueryEquipmentsOnWeapon,
) -> Result<Item, ItemError> {
    let name = weapon.name.clone();
    let description = weapon
        .desc
        .clone()
        .map(|desc_vec| desc_vec.join(&(0xA as char).to_string()));

    let damage;
    let damage_type;
    if let Some(damage_raw) = weapon.damage.as_ref() {
        let name_cloned = name.clone();
        let damage_type_name = damage_raw
            .damage_type
            .as_ref()
            .ok_or(ItemError::named(
                ItemErrorVariant::ParseError("Damage Type".to_string()),
                name.clone(),
            ))?
            .name
            .as_str();
        damage_type = DamageType::try_from(damage_type_name).map_err(|_| {
            ItemError::named(
                ItemErrorVariant::ParseError("Damage Type".to_string()),
                name_cloned,
            )
        })?;
        if damage_raw.damage_dice.chars().find(|v| *v == 'd').is_some() {
            damage = DamageRoll::from_str(&damage_raw.damage_dice, damage_type).ok_or(
                ItemError::named(
                    ItemErrorVariant::ParseError("Damage Roll".to_string()),
                    name.clone(),
                ),
            )?;
        } else {
            let damage_bonus: isize = damage_raw.damage_dice.parse().map_err(|_| {
                ItemError::named(
                    ItemErrorVariant::ParseError("Damage Dice".to_string()),
                    name.clone(),
                )
            })?;
            damage = DamageRoll {
                number: 0,
                dice: 0,
                bonus: damage_bonus,
                damage_type,
            }
        }
    } else {
        damage_type = DamageType::Force;
        damage = DamageRoll {
            number: 0,
            dice: 0,
            bonus: 0,
            damage_type: DamageType::Force,
        };
    }

    let weapon_type = match weapon.weapon_category.to_lowercase().as_str() {
        "simple" => WeaponType::Simple,
        "martial" => WeaponType::Martial,
        _ => {
            return Err(ItemError::named(
                ItemErrorVariant::ParseError("Weapon Type".to_string()),
                name.clone(),
            ))
        }
    };

    let property_list = weapon
        .properties
        .as_ref()
        .ok_or(ItemError::named(
            ItemErrorVariant::ParseError("Property List".to_string()),
            name.clone(),
        ))?
        .iter()
        .map(|p| p.name.as_str());

    let properties = WeaponProperties::from_list(property_list, damage_type);

    let item_type = ItemType::Weapon(Weapon {
        weapon_type,
        properties,
        damage,
        attack_roll_bonus: 0,
    });

    Ok(Item {
        name,
        description,
        features: vec![],
        item_type,
    })
}

fn generic_item(name: &str, desc: &Option<Vec<String>>) -> Item {
    let description = desc.clone().map(|v| v.join(&(10 as char).to_string()));
    Item {
        name: name.to_string(),
        description,
        item_type: ItemType::Misc,
        features: vec![],
    }
}

#[cfg(test)]
mod item_tests {
    use std::collections::HashMap;

    use crate::{
        graphql::items::new_database,
        rules2014::items::{ArmorCategory, DamageRoll, Item, ItemType},
    };

    #[tokio::test]
    async fn item_tests() {
        let item_database = new_database().await.unwrap();
        shortsword_test(&item_database);
        shield_test(&item_database);
        studded_leather_retrieval(&item_database);
    }

    fn shortsword_test(database: &HashMap<String, Item>) {
        let shortsword = database
            .get("shortsword")
            .expect("Database should've had shortsword");
        assert_eq!(shortsword.name, "Shortsword");
        let weapon_type = match &shortsword.item_type {
            ItemType::Weapon(w) => w,
            ItemType::Misc => panic!("Shortsword should be a weapon, not a misc item"),
            ItemType::Shield => panic!("Shortsword should be a weapon, not a shield"),
            ItemType::Armor(_) => panic!("Shortsword should be a weapon, not armor"),
        };

        assert_eq!(
            weapon_type.damage,
            DamageRoll {
                bonus: 0,
                damage_type: crate::rules2014::items::DamageType::Piercing,
                number: 1,
                dice: 6,
            }
        )
    }

    fn shield_test(database: &HashMap<String, Item>) {
        let shield = database
            .get("shield")
            .expect("Database should have a shield");
        match &shield.item_type {
            ItemType::Weapon(_) => panic!("Shield should be a shield, not a weapon"),
            ItemType::Misc => panic!("Shield should be a shield, not a misc item"),
            ItemType::Armor(_) => panic!("Sheild should be a weapon, not armor"),
            ItemType::Shield => (),
        }
    }

    fn studded_leather_retrieval(database: &HashMap<String, Item>) {
        let v = database
            .get("studded leather armor")
            .expect("Failed to get studded leather");
        assert_eq!(v.name, "Studded Leather Armor");

        let armor = match &v.item_type {
            ItemType::Armor(a) => a,
            _ => panic!("Studded leather armor should be armor!"),
        };

        assert_eq!(armor.ac, 12);
        assert_eq!(armor.category, ArmorCategory::Light);
    }
}
