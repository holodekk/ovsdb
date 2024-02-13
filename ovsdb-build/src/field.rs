use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_quote;

use ovsdb::schema::{Atomic, Column};

use crate::{name_to_ident, Attributes};

fn atomic_to_native_type(atomic: &Atomic) -> syn::Type {
    match atomic {
        Atomic::Boolean => parse_quote! { bool },
        Atomic::Integer => parse_quote! { i64 },
        Atomic::Real => parse_quote! { f64 },
        Atomic::String => parse_quote! { String },
        Atomic::Uuid => parse_quote! { ovsdb::protocol::Uuid },
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Atomic(Atomic),
    Enum(String, Atomic),
    Map(Atomic, Atomic),
    Optional(Box<Kind>),
    Set(Box<Kind>),
}

impl Kind {
    pub fn to_native_type(&self) -> syn::Type {
        match self {
            Self::Atomic(a) => {
                let kind = atomic_to_native_type(a);
                parse_quote! { #kind }
            }
            Self::Enum(name, _) => {
                let enum_name = super::name_to_ident(name);
                parse_quote! { #enum_name }
            }
            Self::Map(k, v) => {
                let key_kind = atomic_to_native_type(k);
                let value_kind = atomic_to_native_type(v);
                parse_quote! { std::collections::BTreeMap<#key_kind, #value_kind> }
            }
            Self::Optional(v) => {
                let value = v.to_native_type();
                parse_quote! { Option<#value> }
            }
            Self::Set(v) => {
                let value = v.to_native_type();
                parse_quote! { Vec<#value> }
            }
        }
    }

    pub fn to_ovsdb_type(&self) -> syn::Type {
        match self {
            Self::Atomic(a) => {
                let kind = atomic_to_native_type(a);
                parse_quote! { #kind }
            }
            Self::Enum(name, _) => {
                let enum_name = super::name_to_ident(name);
                parse_quote! { #enum_name }
            }
            Self::Map(k, v) => {
                let key_kind = atomic_to_native_type(k);
                let value_kind = atomic_to_native_type(v);
                parse_quote! { ovsdb::protocol::Map<#key_kind, #value_kind> }
            }
            Self::Optional(v) => {
                let value = v.to_ovsdb_type();
                parse_quote! { ovsdb::protocol::Optional<#value> }
            }
            Self::Set(v) => {
                let value = v.to_ovsdb_type();
                if matches!(**v, Self::Atomic(Atomic::Uuid)) {
                    parse_quote! { ovsdb::protocol::UuidSet }
                } else {
                    parse_quote! { ovsdb::protocol::Set<#value> }
                }
            }
        }
    }

    pub fn from_column(column: &Column) -> Self {
        let mut field_kind = Self::Atomic(column.kind.key.kind.clone());

        if column.kind.is_enum() {
            field_kind = Self::Enum(
                super::str_to_name(&column.name),
                column.kind.key.kind.clone(),
            );
        }

        if !column.kind.is_scalar() {
            if column.kind.is_optional() {
                field_kind = Self::Optional(Box::new(field_kind));
            } else if column.kind.is_set() {
                field_kind = Self::Set(Box::new(field_kind));
            } else if column.kind.is_map() {
                let key_kind = &column.kind.key.kind;
                let value_kind = &column.kind.value.as_ref().unwrap().kind;
                field_kind = Self::Map(key_kind.clone(), value_kind.clone());
            }
        }

        field_kind
    }
}

#[derive(Debug)]
pub(crate) struct Field {
    ident: syn::Ident,
    ty: syn::Type,
    attributes: Attributes,
}

impl Field {
    pub fn new<T>(name: T, ty: syn::Type) -> Self
    where
        T: AsRef<str>,
    {
        let mut attributes = Attributes::default();
        let ident = match name.as_ref() {
            "type" => {
                attributes.add("#[serde(rename = \"type\")]");
                name_to_ident("kind")
            }
            _ => name_to_ident(name),
        };

        Self {
            ident,
            ty,
            attributes,
        }
    }

    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    fn ty(&self) -> &syn::Type {
        &self.ty
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident();
        let ty = self.ty();
        let attributes = self.attributes();
        tokens.extend(quote! {
            #(#attributes)*
            #ident: #ty
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::prelude::*;

    fn test_struct(v: &Field) -> String {
        let f: syn::File = syn::parse_quote! { struct Test { #v } };
        let mut buffer = Vec::new();
        buffer
            .write_all(prettyplease::unparse(&f).as_bytes())
            .unwrap();
        String::from_utf8(buffer).unwrap()
    }

    #[test]
    fn test_field_boolean() {
        let native_field = Field::new("test", Kind::Atomic(Atomic::Boolean).to_native_type());
        let ovsdb_field = Field::new("test", Kind::Atomic(Atomic::Boolean).to_ovsdb_type());
        let expected = "struct Test {\n    test: bool,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_integer() {
        let native_field = Field::new("test", Kind::Atomic(Atomic::Integer).to_native_type());
        let ovsdb_field = Field::new("test", Kind::Atomic(Atomic::Integer).to_ovsdb_type());
        let expected = "struct Test {\n    test: i64,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_real() {
        let native_field = Field::new("test", Kind::Atomic(Atomic::Real).to_native_type());
        let ovsdb_field = Field::new("test", Kind::Atomic(Atomic::Real).to_ovsdb_type());
        let expected = "struct Test {\n    test: f64,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_string() {
        let native_field = Field::new("test", Kind::Atomic(Atomic::String).to_native_type());
        let ovsdb_field = Field::new("test", Kind::Atomic(Atomic::String).to_ovsdb_type());
        let expected = "struct Test {\n    test: String,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_uuid() {
        let native_field = Field::new("test", Kind::Atomic(Atomic::Uuid).to_native_type());
        let ovsdb_field = Field::new("test", Kind::Atomic(Atomic::Uuid).to_ovsdb_type());
        let expected = "struct Test {\n    test: ovsdb::protocol::Uuid,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_enum() {
        let native_field = Field::new(
            "test",
            Kind::Enum("Test".to_string(), Atomic::String).to_native_type(),
        );
        let ovsdb_field = Field::new(
            "test",
            Kind::Enum("Test".to_string(), Atomic::String).to_ovsdb_type(),
        );
        let expected = "struct Test {\n    test: Test,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_map() {
        let native_field = Field::new(
            "test",
            Kind::Map(Atomic::String, Atomic::Integer).to_native_type(),
        );
        let ovsdb_field = Field::new(
            "test",
            Kind::Map(Atomic::String, Atomic::Integer).to_ovsdb_type(),
        );
        let expected_native =
            "struct Test {\n    test: std::collections::BTreeMap<String, i64>,\n}\n";
        let expected_ovsdb = "struct Test {\n    test: ovsdb::protocol::Map<String, i64>,\n}\n";

        assert_eq!(&test_struct(&native_field), expected_native);
        assert_eq!(&test_struct(&ovsdb_field), expected_ovsdb);
    }

    #[test]
    fn test_field_optional() {
        let native_field = Field::new(
            "test",
            Kind::Optional(Box::new(Kind::Atomic(Atomic::Uuid))).to_native_type(),
        );
        let ovsdb_field = Field::new(
            "test",
            Kind::Optional(Box::new(Kind::Atomic(Atomic::Uuid))).to_ovsdb_type(),
        );
        let expected_native = "struct Test {\n    test: Option<ovsdb::protocol::Uuid>,\n}\n";
        let expected_ovsdb =
            "struct Test {\n    test: ovsdb::protocol::Optional<ovsdb::protocol::Uuid>,\n}\n";

        assert_eq!(&test_struct(&native_field), expected_native);
        assert_eq!(&test_struct(&ovsdb_field), expected_ovsdb);
    }

    #[test]
    fn test_field_set() {
        let native_field = Field::new(
            "test",
            Kind::Set(Box::new(Kind::Atomic(Atomic::String))).to_native_type(),
        );
        let ovsdb_field = Field::new(
            "test",
            Kind::Set(Box::new(Kind::Atomic(Atomic::String))).to_ovsdb_type(),
        );
        let expected_native = "struct Test {\n    test: Vec<String>,\n}\n";
        let expected_ovsdb = "struct Test {\n    test: ovsdb::protocol::Set<String>,\n}\n";

        assert_eq!(&test_struct(&native_field), expected_native);
        assert_eq!(&test_struct(&ovsdb_field), expected_ovsdb);
    }

    #[test]
    fn test_field_uuid_set() {
        let native_field = Field::new(
            "test",
            Kind::Set(Box::new(Kind::Atomic(Atomic::Uuid))).to_native_type(),
        );
        let ovsdb_field = Field::new(
            "test",
            Kind::Set(Box::new(Kind::Atomic(Atomic::Uuid))).to_ovsdb_type(),
        );
        let expected_native = "struct Test {\n    test: Vec<ovsdb::protocol::Uuid>,\n}\n";
        let expected_ovsdb = "struct Test {\n    test: ovsdb::protocol::UuidSet,\n}\n";

        assert_eq!(&test_struct(&native_field), expected_native);
        assert_eq!(&test_struct(&ovsdb_field), expected_ovsdb);
    }
}
