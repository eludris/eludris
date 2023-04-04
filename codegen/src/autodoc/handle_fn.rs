use std::collections::HashMap;

use proc_macro::{Span, TokenStream};
use syn::{spanned::Spanned, Error, FnArg, ItemFn, Lit, Meta, NestedMeta, Pat, ReturnType, Type};

use super::{
    models::{ItemInfo, ParamInfo, RouteInfo},
    utils::{display_path_segment, get_doc, get_type},
};

pub fn handle_fn(attr: TokenStream, item: ItemFn) -> Result<(ItemInfo, String), Error> {
    let name = item.sig.ident.to_string();
    let doc = get_doc(&item.attrs)?;
    let base = if attr.is_empty() {
        "".to_string()
    } else {
        let attr: NestedMeta = syn::parse(attr)?;
        if let NestedMeta::Lit(Lit::Str(lit)) = attr {
            lit.value()
        } else {
            return Err(Error::new(attr.span(), "Invalid attribute args"));
        }
    };

    let attr = item
        .attrs
        .iter()
        .find(|a| {
            a.path.is_ident("get")
                || a.path.is_ident("post")
                || a.path.is_ident("patch")
                || a.path.is_ident("delete")
                || a.path.is_ident("push")
        })
        .ok_or_else(|| Error::new(item.span(), "Could not find rocket method attribute"))?;
    let method = attr
        .path
        .get_ident()
        .expect("Ident removed itself")
        .to_string()
        .to_uppercase();
    let return_type = match item.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(get_type(&ty)?),
    };

    let mut params = HashMap::new();
    for param in item.sig.inputs {
        // But they're called parameters tho :lol:
        if let FnArg::Typed(param) = param {
            if let Pat::Ident(ident) = *param.pat {
                let name = ident.ident.to_string();
                if let Type::Path(ty) = *param.ty {
                    let param_type =
                        display_path_segment(ty.path.segments.last().ok_or_else(|| {
                            Error::new(ty.path.span(), "Cannot extract type from field")
                        })?)?;
                    params.insert(name, param_type);
                } else {
                    // we still want these as guards
                    params.insert(name, "Guard".to_string());
                }
            }
        }
    }

    let mut metas = if let Meta::List(metas) = attr.parse_meta()? {
        metas.nested.into_iter()
    } else {
        return Err(Error::new(attr.span(), "Cannot parse rocket macro"));
    };

    let route = if let NestedMeta::Lit(Lit::Str(route)) = metas
        .next()
        .ok_or_else(|| Error::new(attr.span(), "Could not find route in rocket macro"))?
    {
        route.value()
    } else {
        return Err(Error::new(
            attr.span(),
            "Could not find route in rocket macro",
        ));
    };

    let mut path_params = vec![];
    let mut query_params = vec![];

    let (path, query) = route.split_once('?').unwrap_or((&route, ""));
    for segment in path.split('/') {
        // if it's in the `<name>` format, we want it's type
        if segment.starts_with('<') && segment.ends_with('>') {
            let name = segment[1..segment.len() - 1].to_string();
            let param_type = params.remove(&name).ok_or_else(|| {
                Error::new(
                    Span::call_site().into(),
                    format!("Cannot find type of path param {}", name),
                )
            })?;
            path_params.push(ParamInfo { param_type, name });
        }
    }
    for param in query.split('&') {
        if param.starts_with('<') && param.ends_with('>') {
            let name = param[1..param.len() - 1].to_string();
            let param_type = params.remove(&name).ok_or_else(|| {
                Error::new(
                    Span::call_site().into(),
                    format!("Cannot find type of query param{}", name),
                )
            })?;
            query_params.push(ParamInfo { param_type, name });
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
                    body_type = Some(params.remove(&value[1..value.len() - 1]).unwrap());
                } // rocket should handle other cases for us
            }
        }
    }

    Ok((
        ItemInfo::Route(RouteInfo {
            name: name.clone(),
            method,
            route,
            doc,
            path_params,
            query_params,
            body_type,
            return_type,
            guards: params.into_keys().collect(),
        }),
        name,
    ))
}
