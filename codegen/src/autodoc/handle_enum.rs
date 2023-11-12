use lazy_static::lazy_static;
use regex::Regex;
use syn::{spanned::Spanned, Error, Fields, ItemEnum, Lit, Meta, NestedMeta, Variant};

use super::{
    models::{EnumInfo, EnumVariant, Item},
    utils::{get_doc, get_field_infos, get_type},
};

pub fn handle_enum(_: &[NestedMeta], item: ItemEnum) -> Result<Item, Error> {
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
        variants.push(get_variant(variant, &rename_all)?);
    }

    Ok(Item::Enum(EnumInfo {
        content,
        tag,
        untagged,
        variants,
    }))
}
fn get_variant(variant: Variant, case: &Option<String>) -> Result<EnumVariant, syn::Error> {
    lazy_static! {
        static ref SNAKE_REPLACE: Regex = Regex::new(r"(\S)([A-Z])").unwrap();
    }

    let doc = get_doc(&variant.attrs)?;
    let raw_name = variant.ident.to_string();
    let name = match case.as_deref() {
        Some("SCREAMING_SNAKE_CASE") => SNAKE_REPLACE
            .replace_all(&raw_name, "${1}_$2")
            .to_uppercase(),
        Some("UPPERCASE") => raw_name.to_uppercase(),
        Some("lowercase") => raw_name.to_lowercase(),
        Some(s) => {
            return Err(syn::Error::new(
                variant.span(),
                format!("Unknown rename_all case: {}", s),
            ))
        }
        None => raw_name,
    };
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
        Fields::Named(struct_fields) => EnumVariant::Object {
            name,
            doc,
            fields: get_field_infos(struct_fields.named.iter())?,
        },
    })
}
