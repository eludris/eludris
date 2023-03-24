use proc_macro::TokenStream;
use syn::{spanned::Spanned, Error, Fields, ItemEnum, Lit, Meta, NestedMeta, Variant};

use super::{
    models::{EnumInfo, EnumVariant, ItemInfo, StructInfo},
    utils::{get_doc, get_field_infos, get_type},
};

pub fn handle_enum(attr: TokenStream, item: ItemEnum) -> Result<(ItemInfo, String), Error> {
    if !attr.is_empty() {
        return Err(Error::new(
            syn::parse::<NestedMeta>(attr)?.span(),
            "Struct items expect no attribute args",
        ));
    }

    let name = item.ident.to_string();
    let doc = get_doc(&item.attrs)?;
    let mut rename_all = None;
    let mut tag = None;
    let mut untagged = false;
    let mut content = None;

    for attr in item.attrs.iter().filter(|a| a.path.is_ident("serde")) {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            for meta in meta.nested {
                match meta {
                    NestedMeta::Meta(Meta::NameValue(meta)) => {
                        if let Some(ident) = meta.path.get_ident() {
                            match ident.to_string().as_str() {
                                "rename_all" => {
                                    if let Lit::Str(lit) = meta.lit {
                                        rename_all = Some(lit.value());
                                    }
                                }
                                "tag" => {
                                    if let Lit::Str(lit) = meta.lit {
                                        tag = Some(lit.value());
                                    }
                                }
                                "content" => {
                                    if let Lit::Str(lit) = meta.lit {
                                        content = Some(lit.value());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    NestedMeta::Meta(Meta::Path(path)) => {
                        if path.is_ident("untagged") {
                            untagged = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut variants = vec![];
    for variant in item.variants {
        variants.push(get_variant(variant)?);
    }

    Ok((
        ItemInfo::Enum(EnumInfo {
            name: name.clone(),
            doc,
            content,
            tag,
            untagged,
            rename_all,
            variants,
        }),
        name,
    ))
}
fn get_variant(variant: Variant) -> Result<EnumVariant, syn::Error> {
    let doc = get_doc(&variant.attrs)?;
    let name = variant.ident.to_string();
    Ok(match variant.fields {
        Fields::Unit => EnumVariant::Unit { name, doc },
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() > 1 {
                return Err(Error::new(
                    fields.span(),
                    "Cannot document tuple enum variants with more than one field",
                ));
            }
            let field = fields.unnamed.first().ok_or_else(|| {
                Error::new(
                    fields.span(),
                    "Tuple enum variants must have at least one field",
                )
            })?;
            EnumVariant::Tuple {
                name,
                doc,
                field_type: get_type(&field.ty)?,
            }
        }
        Fields::Named(struct_fields) => EnumVariant::Struct(StructInfo {
            name,
            doc,
            fields: get_field_infos(struct_fields.named.iter())?,
        }),
    })
}
