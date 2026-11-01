use crate::{
    graphql::{classes::ClassError, GraphqlError, GraphqlProvider},
    rules2014::{
        class::ItemCategory,
        features::PresentedOption,
        items::{Item, ItemType, WeaponType},
    },
    DataProvider,
};

use super::class_query::ClassQueryClass;

type EquipmentList = Vec<PresentedOption<Vec<(ItemCategory, usize)>>>;
pub async fn equipment(
    response_data: &ClassQueryClass,
    provider: &GraphqlProvider,
) -> Result<EquipmentList, ClassError> {
    Ok(equipment_given(response_data, provider)
        .await?
        .into_iter()
        .chain(equipment_options(response_data, provider).await?)
        .collect::<Vec<_>>())
}

// equipment that's simply given. e.g. "A holy symbol and 2 daggers."
async fn equipment_given(
    response_data: &ClassQueryClass,
    provider: &GraphqlProvider,
) -> Result<EquipmentList, ClassError> {
    type Equipment = super::class_query::ClassQueryClassStartingEquipmentEquipment;
    let name = &response_data.name;
    let equipment_names = response_data
        .starting_equipment
        .as_ref()
        .ok_or(ClassError::parse_error("starting equipment", name))?
        .iter()
        .map(|z| {
            let quantity: usize = z
                .quantity
                .try_into()
                .map_err(|_| ClassError::parse_error("starting equipment quantity", name))?;
            let unparsed_equipment = z.equipment.as_ref().ok_or(ClassError::parse_error(
                "starting equipment nonreal equipment",
                name,
            ))?;
            let name = match unparsed_equipment {
                Equipment::Armor(a) => a.name.clone(),
                Equipment::Weapon(w) => w.name.clone(),
                Equipment::Tool(t) => t.name.clone(),
                Equipment::Gear(g) => g.name.clone(),
                Equipment::Pack(p) => p.name.clone(),
                Equipment::Ammunition(t) => t.name.clone(),
                Equipment::Vehicle(v) => v.name.clone(),
            };
            Ok::<(String, usize), ClassError>((name, quantity))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut equipment = Vec::with_capacity(equipment_names.len());
    for (equipment_name, equipment_count) in equipment_names {
        let item = provider
            .get_item(&equipment_name)
            .await
            .map_err(|v| match v {
                GraphqlError::Item(item_err) => item_err.into(),
                _ => ClassError::parse_error("Item Retrieval Error", name),
            })?;
        equipment.push(PresentedOption::Base(vec![(
            ItemCategory::Item(item),
            equipment_count,
        )]));
    }

    Ok(equipment)
}

// equipment you have the option to take, e.g. "A shortsword or any simple weapon"
async fn equipment_options(
    response_data: &ClassQueryClass,
    provider: &GraphqlProvider,
) -> Result<EquipmentList, ClassError> {
    use super::class_query::ClassQueryClassStartingEquipmentOptionsFrom::{
        EquipmentCategorySet, EquipmentOptionSet,
    };
    let equipment_option_list = match response_data.starting_equipment_options.as_ref() {
        Some(v) => v,
        None => return Ok(vec![]),
    }
    .iter();

    let mut items = Vec::with_capacity(10);

    for equipment_option in equipment_option_list {
        let item_list = match &equipment_option.from {
            EquipmentCategorySet(category_set) => {
                PresentedOption::Base(category_from_str(&category_set.equipment_category.name))
            }
            EquipmentOptionSet(option_set) => parse_items(option_set, provider).await?,
        };
        items.push(item_list)
    }
    Ok(items)
}

fn category_from_str(name: &str) -> Vec<(ItemCategory, usize)> {
    match name.to_lowercase().as_str() {
        "simple melee weapons" => vec![(ItemCategory::Weapon(WeaponType::Simple), 1)],
        "simple weapons" => vec![(ItemCategory::Weapon(WeaponType::Simple), 1)],
        "martial weapons" => vec![(ItemCategory::Weapon(WeaponType::Simple), 1)],
        o => vec![(
            ItemCategory::Item(Item {
                name: o.to_string(),
                description: None,
                item_type: ItemType::Misc,
                features: vec![],
            }),
            1,
        )],
    }
}

type OptionSet =
    super::class_query::ClassQueryClassStartingEquipmentOptionsFromOnEquipmentOptionSet;
type ItemChoice =
    super::class_query::ClassQueryClassStartingEquipmentOptionsFromOnEquipmentOptionSetOptions;
async fn parse_items(
    option_set: &OptionSet,
    provider: &GraphqlProvider,
) -> Result<PresentedOption<Vec<(ItemCategory, usize)>>, ClassError> {
    let mut items = Vec::with_capacity(option_set.options.len());

    for option in option_set.options.iter() {
        let item = match option {
            ItemChoice::EquipmentCategoryChoiceOption(category) => {
                category_from_str(&category.choice.from.equipment_category.name)
            }
            ItemChoice::CountedReferenceOption(input_items) => {
                counted_refrence_option(input_items, provider).await?
            }
            ItemChoice::MultipleItemsOption(input_items) => {
                multiple_items(&input_items.items, provider).await?
            }
        };
        items.push(item)
    }
    Ok(PresentedOption::Choice(items))
}

type CountedReferenceOption = super::class_query::ClassQueryClassStartingEquipmentOptionsFromOnEquipmentOptionSetOptionsOnCountedReferenceOption;
async fn counted_refrence_option(
    item: &CountedReferenceOption,
    provider: &GraphqlProvider,
) -> Result<Vec<(ItemCategory, usize)>, ClassError> {
    provider
        .get_item(&item.of.name)
        .await
        .map_err(|v| match v {
            GraphqlError::Item(item_err) => item_err.into(),
            _ => ClassError::parse_error("Item Retrieval Error", &item.of.name),
        })
        .and_then(|v| {
            item.count
                .try_into()
                .map_err(|_| ClassError::parse_error("counted ref item count", &item.of.name))
                .map(|z| (v, z))
        })
        .map(|(v, c)| vec![(ItemCategory::Item(v), c)])
}

type SingleItem = super::class_query::ClassQueryClassStartingEquipmentOptionsFromOnEquipmentOptionSetOptionsOnMultipleItemsOptionItems;
async fn multiple_items(
    items: &Vec<SingleItem>,
    provider: &GraphqlProvider,
) -> Result<Vec<(ItemCategory, usize)>, ClassError> {
    let mut output = Vec::with_capacity(items.len());
    for item in items {
        let new_item = match item {
            SingleItem::EquipmentCategoryChoiceOption(o) => {
                category_from_str(&o.choice.from.equipment_category.name)
                    .into_iter()
                    .next()
                    .ok_or(ClassError::parse_error(
                        "equipment cat choice option null category",
                        &o.choice.from.equipment_category.name,
                    ))?
            }
            SingleItem::CountedReferenceOption(o) => {
                let count: usize = o
                    .count
                    .try_into()
                    .map_err(|_| ClassError::parse_error("single item count", &o.of.name))?;
                let constructed_item =
                    provider.get_item(&o.of.name).await.map_err(|v| match v {
                        GraphqlError::Item(i) => ClassError::ItemRetrieval(i),
                        _ => ClassError::parse_error("item retrieval error", &o.of.name),
                    })?;

                (ItemCategory::Item(constructed_item), count)
            }
        };
        output.push(new_item)
    }
    Ok(output)
}
