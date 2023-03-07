mod models;
use std::{env, fs, ops::Not};

use models::{FieldInfo, ItemInfo, StructInfo};
use proc_macro::{Span, TokenStream};
use syn::{spanned::Spanned, Attribute, Error, Item, Lit, Meta, MetaNameValue, Type};

macro_rules! unwrap {
    ($err:expr) => {
        match $err {
            Ok(res) => res,
            Err(err) => return err.to_compile_error().into(),
        }
    };
}

#[proc_macro_attribute]
pub fn autodoc(_: TokenStream, item: TokenStream) -> TokenStream {
    if env::var("ELUDRIS_AUTODOC").is_ok() {
        let item = unwrap!(syn::parse::<Item>(item.clone()));
        let manifest_path = unwrap!(env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| Error::new(item.span(), "Could not find package manifest directory")));
        let package = unwrap!(env::var("CARGO_PKG_NAME")
            .map_err(|_| Error::new(item.span(), "Could not find package name")));
        let (info, name) = match item {
            Item::Fn(item) => {
                println!("fn {}", item.sig.ident);
                todo!()
            }
            Item::Enum(item) => {
                println!("enum {}", item.ident);
                todo!()
            }
            Item::Struct(item) => {
                let name = item.ident.to_string();
                let doc = unwrap!(get_doc(&item.attrs));
                let mut fields = vec![];
                for field in item.fields {
                    if let Type::Path(ty) = &field.ty {
                        let name = unwrap!(field.ident.as_ref().ok_or_else(|| {
                            Error::new(
                                field.span(),
                                "Cannot generate documentation for tuple struct fields",
                            )
                        }))
                        .to_string();
                        let field_type = unwrap!(ty.path.segments.last().ok_or_else(|| {
                            Error::new(ty.path.span(), "Cannot extract type from field")
                        }))
                        .ident
                        .to_string();
                        let doc = unwrap!(get_doc(&field.attrs));
                        let mut flattened = false;
                        for attr in item.attrs.iter().filter(|a| a.path.is_ident("serde")) {
                            if let Ok(Meta::List(meta)) = attr.parse_meta() {
                                for meta in meta.nested {
                                    if let NestedMeta::Meta(Meta::Path(path)) = meta {
                                        if path.is_ident("flatten") {
                                            flattened = true
                                        }
                                    }
                                }
                            }
                        }
                        fields.push(FieldInfo {
                            name,
                            field_type,
                            doc,
                            flattened,
                        })
                    }
                }
                let info = ItemInfo::Struct(StructInfo {
                    name: name.clone(),
                    doc,
                    fields,
                });
                (info, name)
            }
            item => {
                return Error::new(item.span(), "Unsupported item for autodoc")
                    .to_compile_error()
                    .into()
            }
        };
        unwrap!(fs::write(
            format!("{}/../autodoc/{}/{}.json", manifest_path, package, name),
            unwrap!(serde_json::to_string_pretty(&info).map_err(|_| Error::new(
                Span::call_site().into(),
                "Could not convert info into json"
            ))),
        )
        .map_err(|err| Error::new(
            Span::call_site().into(),
            format!("Could not write item info to filesystem: {}", err)
        )));
    };
    item
}

fn get_doc(attrs: &[Attribute]) -> Result<Option<String>, syn::Error> {
    let mut doc = String::new();

    for a in attrs.iter().filter(|a| a.path.is_ident("doc")) {
        let attr: MetaNameValue = match a.parse_meta()? {
            Meta::NameValue(attr) => attr,
            _ => unreachable!(),
        };
        if let Lit::Str(comment) = attr.lit {
            if !doc.is_empty() {
                doc.push('\n');
            };
            let comment = comment.value();
            if let Some(comment) = comment.strip_prefix(' ') {
                doc.push_str(comment);
            } else {
                doc.push_str(&comment);
            };
        }
    }

    Ok(doc.is_empty().not().then_some(doc))
}
