use std::ops::Not;

use syn::{
    spanned::Spanned, AngleBracketedGenericArguments, Attribute, Error, Field, GenericArgument,
    Lit, Meta, MetaNameValue, NestedMeta, PathArguments, PathSegment, Type,
};

use super::models::FieldInfo;

pub fn get_doc(attrs: &[Attribute]) -> Result<Option<String>, Error> {
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

pub fn get_type(ty: &Type) -> Result<String, Error> {
    match ty {
        Type::Path(ty) => Ok(display_path_segment(ty.path.segments.last().ok_or_else(
            || Error::new(ty.span(), "Cannot get last segment of type path"),
        )?)?),
        Type::Reference(ty) => get_type(&ty.elem),
        _ => Err(Error::new(ty.span(), "Cannot document non-path types")),
    }
}

pub fn display_path_segment(segment: &PathSegment) -> Result<String, Error> {
    if segment.ident == "RateLimitedRouteResponse"
        || segment.ident == "Result"
        || segment.ident == "Box"
        || segment.ident == "Custom"
    {
        if let PathArguments::AngleBracketed(args) = &segment.arguments {
            match args.args.first() {
                Some(GenericArgument::Type(Type::Path(ty))) => {
                    if let Some(segment) = ty.path.segments.last() {
                        return display_path_segment(segment);
                    } else {
                        return Err(Error::new(ty.span(), "Cannot extract type from field"));
                    }
                }
                Some(GenericArgument::Type(Type::Tuple(ty))) => {
                    if let Some(ty) = ty.elems.first() {
                        return get_type(ty);
                    } else {
                        // this is valid
                        return Ok("()".to_string());
                    }
                }
                _ => {
                    return Err(Error::new(segment.span(), "Cannot extract type from field"));
                }
            };
        } else {
            return Err(Error::new(segment.span(), "Cannot extract type from field"));
        }
    }

    Ok(match &segment.arguments {
        PathArguments::None => segment.ident.to_string(),
        PathArguments::AngleBracketed(args) => {   
            // convert Vec<T> to T[]
            if segment.ident == "Vec" {
                if let Some(GenericArgument::Type(Type::Path(ty))) = args.args.first() {
                    if let Some(segment) = ty.path.segments.last() {
                        return display_path_segment(segment).map(|s| format!("{}[]", s));
                    } else {
                        return Err(Error::new(ty.span(), "Cannot extract type from field"));
                    }
                }
            }

            let mut arg_strings = vec![];

            args.args.iter().try_for_each(|a| match a {
                GenericArgument::Type(Type::Path(ty)) => {
                    arg_strings.push(display_path_segment(ty.path.segments.last().ok_or_else(
                        || Error::new(ty.path.span(), "Cannot extract type from field"),
                    )?)?);
                    Ok(())
                }
                GenericArgument::Type(Type::Reference(ty)) => {
                    arg_strings.push(get_type(&ty.elem)?);
                    Ok(())
                }
                GenericArgument::Lifetime(_) => Ok(()),
                GenericArgument::Type(Type::Tuple(ty)) => {
                    let types = ty
                        .elems
                        .iter()
                        .filter_map(|t| {
                            let ty = get_type(t).ok()?;
                            if ty == "Status" {
                                None
                            } else {
                                Some(ty)
                            }
                        })
                        .collect::<Vec<_>>();

                    if types.len() > 1 {
                        return Err(Error::new(
                            ty.span(),
                            "Cannot generate documentation for tuple types with more than one non-Status element",
                        ));
                    }
                    if !types.is_empty() {
                        arg_strings.push(types[0].clone());
                    }
                    Ok(())
                }
                _ => Err(Error::new(
                    a.span(),
                    "Cannot generate documentation for non-type generics",
                )),
            })?;

            if !arg_strings.is_empty() {
                format!("{}<{}>", segment.ident, arg_strings.join(", "))
            } else {
                segment.ident.to_string()
            }
        }
        _ => {
            return Err(Error::new(
                segment.span(),
                "Unable to extract type of segment",
            ))
        }
    })
}

pub fn get_field_infos<'a, T: Iterator<Item = &'a Field>>(
    fields: T,
) -> Result<Vec<FieldInfo>, Error> {
    let mut field_infos = vec![];
    'outer: for field in fields {
        let mut name = field
            .ident
            .as_ref()
            .ok_or_else(|| {
                Error::new(
                    field.span(),
                    "Cannot generate documentation for tuple struct fields",
                )
            })?
            .to_string();
        let mut r#type = get_type(&field.ty)?;
        let doc = get_doc(&field.attrs)?;
        let mut flattened = false;
        let mut nullable = false;
        let mut omittable = false;

        // I'm sorry, Torvalds
        for attr in field.attrs.iter().filter(|a| a.path.is_ident("serde")) {
            if let Ok(Meta::List(meta)) = attr.parse_meta() {
                for meta in meta.nested {
                    match meta {
                        NestedMeta::Meta(Meta::Path(path)) => {
                            if path.is_ident("flatten") {
                                flattened = true;
                            } else if path.is_ident("skip") || path.is_ident("skip_serializing") {
                                continue 'outer;
                            }
                        }
                        NestedMeta::Meta(Meta::NameValue(meta)) => {
                            if meta.path.is_ident("skip_serializing_if") {
                                omittable = true;
                                // Strip Option<> from type.
                                if r#type.starts_with("Option<") {
                                    r#type = r#type[7..r#type.len() - 1].to_string();
                                }
                                if let Type::Path(ty) = &field.ty {
                                    if ty.path.segments.last().unwrap().ident == "Option" {
                                        if let PathArguments::AngleBracketed(
                                            AngleBracketedGenericArguments { args, .. },
                                        ) = &ty.path.segments.last().unwrap().arguments
                                        {
                                            if let GenericArgument::Type(Type::Path(ty)) =
                                                args.last().unwrap()
                                            {
                                                if ty.path.segments.last().unwrap().ident
                                                    == "Option"
                                                {
                                                    nullable = true;
                                                    r#type =
                                                        r#type[7..r#type.len() - 1].to_string();
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if meta.path.is_ident("rename") {
                                if let Lit::Str(lit) = meta.lit {
                                    name = lit.value();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Type::Path(ty) = &field.ty {
            if ty.path.segments.last().unwrap().ident == "Option" && !omittable {
                nullable = true;
                r#type = r#type[7..r#type.len() - 1].to_string();
            }
        }
        field_infos.push(FieldInfo {
            name,
            r#type,
            doc,
            flattened,
            nullable,
            omittable,
        })
    }
    Ok(field_infos)
}
