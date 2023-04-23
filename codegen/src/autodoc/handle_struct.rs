use syn::{Error, ItemStruct, NestedMeta};

use super::{
    models::{Item, StructInfo},
    utils::get_field_infos,
};

pub fn handle_struct(_: &[NestedMeta], item: ItemStruct) -> Result<Item, Error> {
    Ok(Item::Struct(StructInfo {
        fields: get_field_infos(item.fields.iter())?,
    }))
}
