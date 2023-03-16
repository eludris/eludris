mod models;
use std::{collections::HashMap, env, fs, ops::Not};

use models::{EnumVariant, FieldInfo, ItemInfo, StructInfo};
use proc_macro::{Span, TokenStream};
use syn::{
    spanned::Spanned, Attribute, Error, Fields, FnArg, GenericArgument, Item, Lit, Meta,
    MetaNameValue, NestedMeta, Pat, PathArguments, PathSegment, ReturnType, Type, Variant,
};

use crate::models::{EnumInfo, PathParamInfo, QueryParamInfo, RouteInfo};

macro_rules! unwrap {
    ($err:expr) => {
        match $err {
            Ok(res) => res,
            Err(err) => return err.to_compile_error().into(),
        }
    };
}

#[proc_macro_attribute]
pub fn autodoc(attr: TokenStream, item: TokenStream) -> TokenStream {
    if env::var("ELUDRIS_AUTODOC").is_ok() {
        let item = unwrap!(syn::parse::<Item>(item.clone()));
        let manifest_path = unwrap!(env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| Error::new(item.span(), "Could not find package manifest directory")));
        let package = unwrap!(env::var("CARGO_PKG_NAME")
            .map_err(|_| Error::new(item.span(), "Could not find package name")));
        let (info, name) = match item {
            Item::Fn(item) => {
                let name = item.sig.ident.to_string();
                let doc = unwrap!(get_doc(&item.attrs));
                let base = if attr.is_empty() {
                    "".to_string()
                } else {
                    let attr: NestedMeta = unwrap!(syn::parse(attr));
                    if let NestedMeta::Lit(Lit::Str(lit)) = attr {
                        lit.value()
                    } else {
                        return Error::new(attr.span(), "Invalid attribute args")
                            .to_compile_error()
                            .into();
                    }
                };
                let attr = unwrap!(item
                    .attrs
                    .iter()
                    .find(|a| {
                        a.path.is_ident("get")
                            || a.path.is_ident("post")
                            || a.path.is_ident("patch")
                            || a.path.is_ident("delete")
                            || a.path.is_ident("push")
                    })
                    .ok_or_else(|| {
                        Error::new(item.span(), "Could not find rocket method attribute")
                    }));
                let return_type = match item.sig.output {
                    ReturnType::Default => None,
                    ReturnType::Type(_, ty) => {
                        if let Type::Path(ty) = *ty {
                            Some(unwrap!(display_path_segment(unwrap!(ty
                                .path
                                .segments
                                .last()
                                .ok_or_else(|| {
                                    Error::new(ty.path.span(), "Cannot extract type from field")
                                })))))
                        } else {
                            return Error::new(ty.span(), "Cannot extract type from field")
                                .to_compile_error()
                                .into();
                        }
                    }
                };
                let mut params = HashMap::new();
                for param in item.sig.inputs {
                    // But they're called parameters tho :lol:
                    if let FnArg::Typed(param) = param {
                        if let Pat::Ident(ident) = *param.pat {
                            let name = ident.ident.to_string();
                            if let Type::Path(ty) = *param.ty {
                                let param_type = unwrap!(display_path_segment(unwrap!(ty
                                    .path
                                    .segments
                                    .last()
                                    .ok_or_else(|| {
                                        Error::new(ty.path.span(), "Cannot extract type from field")
                                    }))));
                                params.insert(name, param_type);
                            } else {
                                // we still want these as guards
                                params.insert(name, "Guard".to_string());
                            }
                        }
                    }
                }

                let mut metas = if let Meta::List(metas) = unwrap!(attr.parse_meta()) {
                    metas.nested.into_iter()
                } else {
                    return Error::new(attr.span(), "Cannot parse rocket macro")
                        .to_compile_error()
                        .into();
                };

                let route = if let NestedMeta::Lit(Lit::Str(route)) = unwrap!(metas
                    .next()
                    .ok_or_else(|| Error::new(attr.span(), "Could not find route in rocket macro")))
                {
                    route.value()
                } else {
                    return Error::new(attr.span(), "Could not find route in rocket macro")
                        .to_compile_error()
                        .into();
                };

                let mut path_params = vec![];
                let mut query_params = vec![];

                let (path, query) = route.split_once('?').unwrap_or((&route, ""));
                for segment in path.split('/') {
                    // if it's in the `<name>` format, we want it's type
                    if segment.starts_with('<') && segment.ends_with('>') {
                        let name = segment[1..segment.len() - 1].to_string();
                        path_params.push(PathParamInfo {
                            param_type: unwrap!(params.remove(&name).ok_or_else(|| Error::new(
                                Span::call_site().into(),
                                format!("Cannot find type of path param{}", name)
                            ))),
                            name,
                        });
                    }
                }
                for param in query.split('&') {
                    if param.starts_with('<') && param.ends_with('>') {
                        let name = param[1..param.len() - 1].to_string();
                        query_params.push(QueryParamInfo {
                            param_type: unwrap!(params.remove(&name).ok_or_else(|| Error::new(
                                Span::call_site().into(),
                                format!("Cannot find type of path param{}", name)
                            ))),
                            name,
                        });
                    }
                }

                let mut route = format!("{}{}", base, route);
                if route != "/" {
                    if let Some(new_route) = route.strip_suffix('/') {
                        route = new_route.to_string();
                    }
                }

                let mut body_type = None;
                for meta in metas {
                    if let NestedMeta::Meta(Meta::NameValue(meta)) = meta {
                        if meta.path.is_ident("data") {
                            if let Lit::Str(lit) = meta.lit {
                                let value = lit.value();
                                // the absence of this should be handled by rocket
                                // also, the format of the lit here is `<name>` because... reasons
                                body_type =
                                    Some(params.remove(&value[1..value.len() - 1]).unwrap());
                            } // rocket should handle other cases for us
                        }
                    }
                }

                (
                    ItemInfo::Route(RouteInfo {
                        name: name.clone(),
                        route,
                        doc,
                        path_params,
                        query_params,
                        body_type,
                        return_type,
                        guards: params.into_keys().collect(),
                    }),
                    name,
                )
            }
            Item::Enum(item) => {
                if !attr.is_empty() {
                    return Error::new(
                        unwrap!(syn::parse::<NestedMeta>(attr)).span(),
                        "Struct items expect no attribute args",
                    )
                    .to_compile_error()
                    .into();
                }
                let name = item.ident.to_string();
                let doc = unwrap!(get_doc(&item.attrs));
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
                    variants.push(unwrap!(get_variant(variant)));
                }
                (
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
                )
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
                        let field_type = unwrap!(display_path_segment(unwrap!(ty
                            .path
                            .segments
                            .last()
                            .ok_or_else(|| {
                                Error::new(ty.path.span(), "Cannot extract type from field")
                            }))));
                        let doc = unwrap!(get_doc(&field.attrs));
                        let mut flattened = false;
                        for attr in field.attrs.iter().filter(|a| a.path.is_ident("serde")) {
                            if let Ok(Meta::List(meta)) = attr.parse_meta() {
                                for meta in meta.nested {
                                    if let NestedMeta::Meta(Meta::Path(path)) = meta {
                                        if path.is_ident("flatten") {
                                            flattened = true;
                                        } else if path.is_ident("skip") {
                                            continue;
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
                    } else {
                        return Error::new(
                            field.span(),
                            "Cannot document non-path typed struct fields",
                        )
                        .to_compile_error()
                        .into();
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
            if let Type::Path(ty) = &field.ty {
                let field_type =
                    display_path_segment(ty.path.segments.last().ok_or_else(|| {
                        Error::new(ty.path.span(), "Cannot extract type from field")
                    })?)?;
                EnumVariant::Tuple {
                    name,
                    doc,
                    field_type,
                }
            } else {
                return Err(Error::new(
                    field.span(),
                    "Cannot document non-path typed struct fields",
                ));
            }
        }
        Fields::Named(struct_fields) => {
            let mut fields = vec![];
            for field in struct_fields.named {
                if let Type::Path(ty) = &field.ty {
                    let name = field
                        .ident
                        .as_ref()
                        .ok_or_else(|| {
                            Error::new(
                                field.span(),
                                "Cannot generate documentation for tuple struct fields",
                            )
                        })?
                        .to_string();
                    let field_type =
                        display_path_segment(ty.path.segments.last().ok_or_else(|| {
                            Error::new(ty.path.span(), "Cannot extract type from field")
                        })?)?;
                    let doc = get_doc(&field.attrs)?;
                    let mut flattened = false;
                    for attr in field.attrs.iter().filter(|a| a.path.is_ident("serde")) {
                        if let Ok(Meta::List(meta)) = attr.parse_meta() {
                            for meta in meta.nested {
                                if let NestedMeta::Meta(Meta::Path(path)) = meta {
                                    if path.is_ident("flatten") {
                                        flattened = true;
                                    } else if path.is_ident("skip") {
                                        continue;
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
                } else {
                    return Err(Error::new(
                        field.span(),
                        "Cannot document non-path typed struct fields",
                    ));
                }
            }
            EnumVariant::Struct(StructInfo { name, doc, fields })
        }
    })
}

fn display_path_segment(segment: &PathSegment) -> Result<String, syn::Error> {
    Ok(match &segment.arguments {
        PathArguments::None => segment.ident.to_string(),
        PathArguments::AngleBracketed(args) => {
            let mut arg_strings = vec![];
            for arg in &args.args {
                if let GenericArgument::Type(Type::Path(ty)) = arg {
                    arg_strings.push(display_path_segment(ty.path.segments.last().ok_or_else(
                        || Error::new(ty.path.span(), "Cannot extract type from field"),
                    )?)?)
                } else {
                    return Err(Error::new(
                        arg.span(),
                        "Cannot generated docummentation for non-type generics",
                    ));
                }
            }

            format!("{}<{}>", segment.ident, arg_strings.join(", "))
        }
        _ => {
            return Err(Error::new(
                segment.span(),
                "Unable to extract type of segment",
            ))
        }
    })
}
