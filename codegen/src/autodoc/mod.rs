mod handle_enum;
mod handle_fn;
mod handle_struct;
mod models;
mod utils;

use std::{env, fs};

use proc_macro::{Span, TokenStream};
use syn::{spanned::Spanned, Error, Item};

pub fn handle_autodoc(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    if env::var("ELUDRIS_AUTODOC").is_ok() {
        let item = syn::parse::<Item>(item.clone())?;
        let manifest_path = env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| Error::new(item.span(), "Could not find package manifest directory"))?;
        let package = env::var("CARGO_PKG_NAME")
            .map_err(|_| Error::new(item.span(), "Could not find package name"))?;

        let (info, name) = match item {
            Item::Fn(item) => handle_fn::handle_fn(attr, item)?,
            Item::Enum(item) => handle_enum::handle_enum(attr, item)?,
            Item::Struct(item) => handle_struct::handle_struct(attr, item)?,
            item => return Err(Error::new(item.span(), "Unsupported item for autodoc")),
        };

        fs::write(
            format!("{}/../autodoc/{}/{}.json", manifest_path, package, name),
            serde_json::to_string_pretty(&info).map_err(|_| {
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
