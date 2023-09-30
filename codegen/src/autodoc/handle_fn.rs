use std::collections::HashMap;

use proc_macro::Span;
use syn::{spanned::Spanned, Error, FnArg, ItemFn, Lit, Meta, NestedMeta, Pat, ReturnType, Type};

use super::{
    models::{Body, Item, ParamInfo, Response, RouteInfo},
    utils::get_type,
};

fn has_rate_limits(ty: &Type) -> Result<bool, Error> {
    if let Type::Path(path) = ty {
        if let Some(item) = path.path.segments.last() {
            if item.ident == "RateLimitedRouteResponse" {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn route_format(ty: String) -> Result<(String, String), Error> {
    if ty.starts_with("Json<") {
        Ok((
            ty[5..ty.len() - 1].to_string(),
            "application/json".to_string(),
        ))
    } else if ty.starts_with("Form<") {
        Ok((
            ty[5..ty.len() - 1].to_string(),
            "multipart/form-data".to_string(),
        ))
    } else if ty == "FetchResponse" {
        Ok((ty, "raw".to_string()))
    } else {
        return Err(Error::new(
            Span::call_site().into(),
            format!("Could not parse route format: {}", ty),
        ));
    }
}

pub fn handle_fn(attrs: &[NestedMeta], item: ItemFn, status_code: u8) -> Result<Item, Error> {
    let mut base = "".to_string();

    for attr in attrs.iter() {
        if let NestedMeta::Lit(Lit::Str(lit)) = attr {
            if !base.is_empty() {
                return Err(Error::new(
                    attr.span(),
                    "Duplicate arguments for route base path",
                ));
            }
            base = lit.value();
        }
    }

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
    let (response, rate_limit) = match item.sig.output {
        ReturnType::Default => (None, false),
        ReturnType::Type(_, ty) => (Some(route_format(get_type(&ty)?)?), has_rate_limits(&ty)?),
    };

    let mut params = HashMap::new();
    for param in item.sig.inputs {
        // But they're called parameters tho :lol:
        if let FnArg::Typed(param) = param {
            if let Pat::Ident(ident) = *param.pat {
                let name = ident.ident.to_string();
                if let Ok(param_type) = get_type(&param.ty) {
                    params.insert(name, param_type);
                } else {
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
            let r#type = params.remove(&name).ok_or_else(|| {
                Error::new(
                    Span::call_site().into(),
                    format!("Cannot find type of path param {}", name),
                )
            })?;
            path_params.push(ParamInfo { r#type, name });
        }
    }
    for param in query.split('&') {
        if param.starts_with('<') && param.ends_with('>') {
            let name = param[1..param.len() - 1].to_string();
            let r#type = params.remove(&name).ok_or_else(|| {
                Error::new(
                    Span::call_site().into(),
                    format!("Cannot find type of query param {}", name),
                )
            })?;
            query_params.push(ParamInfo { r#type, name });
        }
    }

    let mut route = format!("{}{}", base, route);
    if route != "/" {
        if let Some(new_route) = route.strip_suffix('/') {
            route = new_route.to_string();
        }
    }

    let mut body = None;
    for meta in metas {
        if let NestedMeta::Meta(Meta::NameValue(meta)) = meta {
            if meta.path.is_ident("data") {
                if let Lit::Str(lit) = meta.lit {
                    let value = lit.value();
                    // the absence of this should be handled by rocket
                    // also, the format of the lit here is `<name>` because... reasons
                    body = Some(route_format(
                        params.remove(&value[1..value.len() - 1]).unwrap(),
                    )?);
                } // rocket should handle other cases for us
            }
        }
    }

    let requires_auth = match params.get("session").map(|s| s.to_string()).as_deref() {
        Some("TokenAuth") => Some(true),
        Some("Option<TokenAuth>") => Some(false),
        Some(_) => {
            return Err(Error::new(
                Span::call_site().into(),
                "Session parameter must be of type TokenAuth or Option<TokenAuth>",
            ))
        }
        None => None,
    };
    Ok(Item::Route(RouteInfo {
        method,
        route,
        path_params,
        query_params,
        body: body.map(|b| Body {
            r#type: b.0,
            format: b.1,
        }),
        response: response.map(|r| Response {
            r#type: r.0,
            format: r.1,
            status_code,
            rate_limit,
        }),
        requires_auth,
    }))
}
