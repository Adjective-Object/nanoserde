use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::parse::Attribute;
#[cfg(any(feature = "binary", feature = "json"))]
use crate::parse::{Enum, Struct};

macro_rules! l {
    ($target:ident, $line:expr) => {
        $target.push_str($line)
    };

    ($target:ident, $line:expr, $($param:expr),*) => {
        $target.push_str(&::alloc::format!($line, $($param,)*))
    };
}

fn expect_args_len(attr: &Attribute, expected_len: usize) -> Result<(), String> {
    if attr.tokens.len() != expected_len + 1 {
        return Err(format!(
            "Attribute \"{}\" expects {} arguments, found {}",
            attr.tokens[0].as_str(),
            expected_len,
            attr.tokens.len() - 1
        ));
    }
    Ok(())
}

fn expect_args_lens(attr: &Attribute, expected_lens: &[usize]) -> Result<(), String> {
    if !expected_lens
        .iter()
        .any(|expected_len| attr.tokens.len() == expected_len + 1)
    {
        let mut lens = expected_lens
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        let last = lens.pop();
        let lens_msg = lens.join(", ") + " or " + last.unwrap().as_str();

        return Err(format!(
            "Attribute \"{}\" expects {} arguments, found {}",
            attr.tokens[0].as_str(),
            lens_msg,
            attr.tokens.len() - 1
        ));
    }
    Ok(())
}

pub fn validate_attrs(attributes: &[Attribute]) -> Result<(), String> {
    for attr in attributes.iter() {
        if attr.tokens.len() < 1 {
            return Err("Attribute must have at least one token".to_string());
        }

        let attr_name = attr.tokens[0].as_str();
        let result = match attr_name {
            "proxy" => expect_args_len(attr, 1),
            "rename" => expect_args_len(attr, 1),
            "default" => expect_args_lens(attr, &[0, 1]),
            "default_with" => expect_args_len(attr, 1),
            "transparent" => expect_args_len(attr, 0),
            "skip" => expect_args_len(attr, 0),
            _ => {
                return Err(format!(
                    "unrecognized nserde() struct attribute: {:?}",
                    attr_name
                ))
            }
        };

        if let Err(e) = result {
            return Err(e);
        }
    }

    Ok(())
}

pub fn validate_field_attrs(attributes: &[Attribute]) -> Result<(), String> {
    for attr in attributes.iter() {
        if attr.tokens.len() < 1 {
            return Err("Attribute must have at least one token".to_string());
        }

        let attr_name = attr.tokens[0].as_str();
        let result = match attr_name {
            "proxy" => expect_args_len(attr, 1),
            "rename" => expect_args_len(attr, 1),
            "default" => expect_args_lens(attr, &[0, 1]),
            "default_with" => expect_args_len(attr, 1),
            "serialize_none_as_null" => expect_args_len(attr, 0),
            "skip" => expect_args_len(attr, 0),
            _ => {
                return Err(format!(
                    "unrecognized nserde() field attribute: {:?}",
                    attr_name
                ))
            }
        };

        if let Err(e) = result {
            return Err(e);
        }
    }

    Ok(())
}

pub fn attrs_proxy(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0] == "proxy" {
            Some(attr.tokens[1].clone())
        } else {
            None
        }
    })
}

#[cfg(any(feature = "ron", feature = "json"))]
pub fn attrs_rename(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0] == "rename" {
            Some(attr.tokens[1].clone())
        } else {
            None
        }
    })
}

#[cfg(any(feature = "ron", feature = "json"))]
pub fn attrs_default(attributes: &[crate::parse::Attribute]) -> Option<Option<String>> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 1 && attr.tokens[0] == "default" {
            Some(None)
        } else if attr.tokens.len() == 2 && attr.tokens[0] == "default" {
            Some(Some(attr.tokens[1].clone()))
        } else {
            None
        }
    })
}

#[cfg(any(feature = "ron", feature = "json"))]
pub fn attrs_default_with(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0] == "default_with" {
            Some(attr.tokens[1].clone())
        } else {
            None
        }
    })
}

#[cfg(feature = "json")]
pub fn attrs_transparent(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0] == "transparent")
}

#[cfg(any(feature = "json", feature = "ron"))]
pub fn attrs_skip(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0] == "skip")
}

#[cfg(feature = "json")]
pub fn attrs_serialize_none_as_null(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0] == "serialize_none_as_null")
}

#[cfg(any(feature = "binary", feature = "json"))]
pub(crate) fn struct_bounds_strings(struct_: &Struct, bound_name: &str) -> (String, String) {
    let generics: &Vec<_> = &struct_.generics;

    if generics.is_empty() {
        return ("".to_string(), "".to_string());
    }
    let mut generic_w_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_w_bounds += generic
            .full_with_const(&[format!("nanoserde::{}", bound_name).as_str()], true)
            .as_str();
        generic_w_bounds += ", ";
    }
    generic_w_bounds += ">";

    let mut generic_no_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_no_bounds += generic.ident_only().as_str();
        generic_no_bounds += ", ";
    }
    generic_no_bounds += ">";
    return (generic_w_bounds, generic_no_bounds);
}

#[cfg(any(feature = "binary", feature = "json"))]
pub(crate) fn enum_bounds_strings(enum_: &Enum, bound_name: &str) -> (String, String) {
    let generics: &Vec<_> = &enum_.generics;

    if generics.is_empty() {
        return ("".to_string(), "".to_string());
    }
    let mut generic_w_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_w_bounds += generic
            .full_with_const(&[format!("nanoserde::{}", bound_name).as_str()], true)
            .as_str();
        generic_w_bounds += ", ";
    }
    generic_w_bounds += ">";

    let mut generic_no_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_no_bounds += generic.ident_only().as_str();
        generic_no_bounds += ", ";
    }
    generic_no_bounds += ">";
    return (generic_w_bounds, generic_no_bounds);
}
