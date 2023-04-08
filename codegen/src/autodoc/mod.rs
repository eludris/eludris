mod handle_enum;
mod handle_fn;
mod handle_struct;
mod models;
mod utils;

use std::{env, fs};

use proc_macro::{Span, TokenStream};
use syn::{parse::Parser, spanned::Spanned, Error, Item, Lit, Meta, NestedMeta};

use self::{models::ItemInfo, utils::get_doc};

pub fn handle_autodoc(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    if env::var("ELUDRIS_AUTODOC").is_ok() {
        let item = syn::parse::<Item>(item.clone())?;
        let manifest_path = env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| Error::new(item.span(), "Could not find package manifest directory"))?;
        let package = env::var("CARGO_PKG_NAME")
            .map_err(|_| Error::new(item.span(), "Could not find package name"))?;
        let attrs = syn::punctuated::Punctuated::<NestedMeta, syn::Token![,]>::parse_terminated
            .parse(attr)
            .unwrap();
        let attr_span = attrs.span();

        let mut category = None;

        // no drain_filter wa
        let attrs = attrs
            .into_iter()
            .filter_map(|a| {
                if let NestedMeta::Meta(Meta::NameValue(meta)) = &a {
                    if meta.path.is_ident("category") {
                        if let Lit::Str(lit) = &meta.lit {
                            category = Some(lit.value());
                            return None;
                        } else {
                            return Some(Err(Error::new(
                                meta.span(),
                                "Expected a String for item category",
                            )));
                        }
                    }
                }
                Some(Ok(a))
            })
            .collect::<Result<Vec<NestedMeta>, Error>>()?;

        let category = category.ok_or_else(|| {
            Error::new(
                attr_span,
                "Could not find item category in attribute arguments",
            )
        })?;

        let item = match item {
            Item::Fn(item) => {
                let name = item.sig.ident.to_string();
                let doc = get_doc(&item.attrs)?;
                ItemInfo {
                    name,
                    doc,
                    category,
                    package: package.clone(),
                    item: handle_fn::handle_fn(&attrs, item)?,
                }
            }
            Item::Enum(item) => {
                let name = item.ident.to_string();
                let doc = get_doc(&item.attrs)?;
                ItemInfo {
                    name,
                    doc,
                    category,
                    package: package.clone(),
                    item: handle_enum::handle_enum(&attrs, item)?,
                }
            }
            Item::Struct(item) => {
                let name = item.ident.to_string();
                let doc = get_doc(&item.attrs)?;
                ItemInfo {
                    name,
                    doc,
                    category,
                    package: package.clone(),
                    item: handle_struct::handle_struct(&attrs, item)?,
                }
            }
            item => return Err(Error::new(item.span(), "Unsupported item for autodoc")),
        };

        fs::write(
            format!(
                "{}/../autodoc/{}/{}.json",
                manifest_path, package, item.name
            ),
            serde_json::to_string_pretty(&item).map_err(|_| {
                Error::new(Span::call_site().into(), "Could not convert info into json")
            })?,
        )
        .map_err(|err| {
            Error::new(
                Span::call_site().into(),
                format!("Could not write item info to filesystem: {}", err),
            )
        })?;
    };
    Ok(item)
}
