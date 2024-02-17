use syn::{Error, ItemStruct, NestedMeta};

use super::{
    models::{Item, ObjectInfo},
    utils::get_field_infos,
};

pub fn handle_struct(_: &[NestedMeta], item: ItemStruct) -> Result<Item, Error> {
    Ok(Item::Object(ObjectInfo {
        fields: get_field_infos(item.fields.iter())?,
    }))
}
