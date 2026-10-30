use crate::{
    graphql::classes::ClassError,
    rules2014::{class::ItemCategory, features::PresentedOption},
};

use super::class_query::ClassQueryClass;

type EquipmentList = Vec<PresentedOption<Vec<(ItemCategory, usize)>>>;
pub async fn equipment(response_data: &ClassQueryClass) -> Result<EquipmentList, ClassError> {
    Ok(equipment_options(response_data)
        .await?
        .into_iter()
        .chain(equipment_given(response_data).await?)
        .collect::<Vec<_>>())
}

// equipment you have the option to take, e.g. "A shortsword or any simple weapon"
async fn equipment_options(response_data: &ClassQueryClass) -> Result<EquipmentList, ClassError> {
    todo!()
}

// equipment that's simply given. e.g. "A holy symbol and 2 daggers."
async fn equipment_given(response_data: &ClassQueryClass) -> Result<EquipmentList, ClassError> {
    todo!()
}
