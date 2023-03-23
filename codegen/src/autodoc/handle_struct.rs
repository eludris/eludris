use proc_macro::TokenStream;
use syn::{spanned::Spanned, Error, ItemStruct, NestedMeta};

use super::{
    models::{ItemInfo, StructInfo},
    utils::{get_doc, get_field_infos},
};

pub fn handle_struct(attr: TokenStream, item: ItemStruct) -> Result<(ItemInfo, String), Error> {
    if !attr.is_empty() {
        return Err(Error::new(
            syn::parse::<NestedMeta>(attr)?.span(),
            "Struct items expect no attribute args",
        ));
    }

    let name = item.ident.to_string();
    let doc = get_doc(&item.attrs)?;
    Ok((
        ItemInfo::Struct(StructInfo {
            name: name.clone(),
            doc,
            fields: get_field_infos(item.fields.iter())?,
        }),
        name,
    ))
}
