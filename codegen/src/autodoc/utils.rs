use std::ops::Not;

use syn::{
    spanned::Spanned, Attribute, Error, Field, GenericArgument, Lit, Meta, MetaNameValue,
    NestedMeta, PathArguments, PathSegment, Type,
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
    if let Type::Path(ty) = ty {
        Ok(display_path_segment(ty.path.segments.last().ok_or_else(
            || Error::new(ty.span(), "Cannot get last segment of type path"),
        )?)?)
    } else {
        Err(Error::new(ty.span(), "Cannot document non-path types"))
    }
}

pub fn display_path_segment(segment: &PathSegment) -> Result<String, Error> {
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

pub fn get_field_infos<'a, T: Iterator<Item = &'a Field>>(
    fields: T,
) -> Result<Vec<FieldInfo>, Error> {
    let mut field_infos = vec![];
    for field in fields {
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
        let field_type = get_type(&field.ty)?;
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
        field_infos.push(FieldInfo {
            name,
            field_type,
            doc,
            flattened,
        })
    }
    Ok(field_infos)
}
