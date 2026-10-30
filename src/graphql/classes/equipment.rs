use crate::{
    graphql::{classes::ClassError, GraphqlError, GraphqlProvider},
    rules2014::{class::ItemCategory, features::PresentedOption},
    DataProvider,
};

use super::class_query::ClassQueryClass;

type EquipmentList = Vec<PresentedOption<Vec<(ItemCategory, usize)>>>;
pub async fn equipment(
    response_data: &ClassQueryClass,
    provider: GraphqlProvider,
) -> Result<EquipmentList, ClassError> {
    Ok(equipment_options(response_data)
        .await?
        .into_iter()
        .chain(equipment_given(response_data, provider).await?)
        .collect::<Vec<_>>())
}

// equipment you have the option to take, e.g. "A shortsword or any simple weapon"
async fn equipment_options(response_data: &ClassQueryClass) -> Result<EquipmentList, ClassError> {
    todo!()
}

// equipment that's simply given. e.g. "A holy symbol and 2 daggers."
async fn equipment_given(
    response_data: &ClassQueryClass,
    provider: GraphqlProvider,
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
