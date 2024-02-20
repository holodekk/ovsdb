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
    pub(crate) fn to_native_type(&self) -> syn::Type {
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

    pub(crate) fn to_ovsdb_type(&self) -> syn::Type {
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

    pub(crate) fn from_column(column: &Column) -> Self {
        let mut field_kind = Self::Atomic(column.kind().key().kind());

        if column.kind().is_enum() {
            field_kind = Self::Enum(
                super::str_to_name(column.name()),
                column.kind().key().kind(),
            );
        }

        if !column.kind().is_scalar() {
            if column.kind().is_optional() {
                field_kind = Self::Optional(Box::new(field_kind));
            } else if column.kind().is_set() {
                field_kind = Self::Set(Box::new(field_kind));
            } else if column.kind().is_map() {
                let key_kind = &column.kind().key().kind();
                let value_kind = &column
                    .kind()
                    .value()
                    .as_ref()
                    .expect("column value kind")
                    .kind();
                field_kind = Self::Map(*key_kind, *value_kind);
            }
        }

        field_kind
    }
}

#[derive(Debug)]
pub(crate) struct Field {
    ident: syn::Ident,
    kind: Kind,
    ty: syn::Type,
    attributes: Attributes,
}

impl Field {
    fn new<T>(name: T, kind: Kind, ty: syn::Type) -> Self
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
            kind,
            ty,
            attributes,
        }
    }

    pub(crate) fn native<T>(name: T, kind: &Kind) -> Self
    where
        T: AsRef<str>,
    {
        Self::new(name, kind.clone(), kind.to_native_type())
    }

    pub(crate) fn ovsdb<T>(name: T, kind: &Kind) -> Self
    where
        T: AsRef<str>,
    {
        Self::new(name, kind.clone(), kind.to_ovsdb_type())
    }

    /// Returns a reference to the ident of this [`Field`].
    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub(crate) fn kind(&self) -> &Kind {
        &self.kind
    }

    pub(crate) fn ty(&self) -> &syn::Type {
        &self.ty
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    pub(crate) fn is_atomic(&self) -> bool {
        matches!(self.kind(), Kind::Atomic(_))
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
            .expect("parsed");
        String::from_utf8(buffer).expect("utf8 string")
    }

    #[test]
    fn test_field_boolean() {
        let native_field = Field::native("test", &Kind::Atomic(Atomic::Boolean));
        let ovsdb_field = Field::ovsdb("test", &Kind::Atomic(Atomic::Boolean));
        let expected = "struct Test {\n    test: bool,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_integer() {
        let native_field = Field::native("test", &Kind::Atomic(Atomic::Integer));
        let ovsdb_field = Field::ovsdb("test", &Kind::Atomic(Atomic::Integer));
        let expected = "struct Test {\n    test: i64,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_real() {
        let native_field = Field::native("test", &Kind::Atomic(Atomic::Real));
        let ovsdb_field = Field::ovsdb("test", &Kind::Atomic(Atomic::Real));
        let expected = "struct Test {\n    test: f64,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_string() {
        let native_field = Field::native("test", &Kind::Atomic(Atomic::String));
        let ovsdb_field = Field::ovsdb("test", &Kind::Atomic(Atomic::String));
        let expected = "struct Test {\n    test: String,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_uuid() {
        let native_field = Field::native("test", &Kind::Atomic(Atomic::Uuid));
        let ovsdb_field = Field::ovsdb("test", &Kind::Atomic(Atomic::Uuid));
        let expected = "struct Test {\n    test: ovsdb::protocol::Uuid,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_enum() {
        let native_field = Field::native("test", &Kind::Enum("Test".to_string(), Atomic::String));
        let ovsdb_field = Field::ovsdb("test", &Kind::Enum("Test".to_string(), Atomic::String));
        let expected = "struct Test {\n    test: Test,\n}\n";

        assert_eq!(&test_struct(&native_field), expected);
        assert_eq!(&test_struct(&ovsdb_field), expected);
    }

    #[test]
    fn test_field_map() {
        let native_field = Field::native("test", &Kind::Map(Atomic::String, Atomic::Integer));
        let ovsdb_field = Field::ovsdb("test", &Kind::Map(Atomic::String, Atomic::Integer));
        let expected_native =
            "struct Test {\n    test: std::collections::BTreeMap<String, i64>,\n}\n";
        let expected_ovsdb = "struct Test {\n    test: ovsdb::protocol::Map<String, i64>,\n}\n";

        assert_eq!(&test_struct(&native_field), expected_native);
        assert_eq!(&test_struct(&ovsdb_field), expected_ovsdb);
    }

    #[test]
    fn test_field_optional() {
        let native_field = Field::native(
            "test",
            &Kind::Optional(Box::new(Kind::Atomic(Atomic::Uuid))),
        );
        let ovsdb_field = Field::ovsdb(
            "test",
            &Kind::Optional(Box::new(Kind::Atomic(Atomic::Uuid))),
        );
        let expected_native = "struct Test {\n    test: Option<ovsdb::protocol::Uuid>,\n}\n";
        let expected_ovsdb =
            "struct Test {\n    test: ovsdb::protocol::Optional<ovsdb::protocol::Uuid>,\n}\n";

        assert_eq!(&test_struct(&native_field), expected_native);
        assert_eq!(&test_struct(&ovsdb_field), expected_ovsdb);
    }

    #[test]
    fn test_field_set() {
        let native_field =
            Field::native("test", &Kind::Set(Box::new(Kind::Atomic(Atomic::String))));
        let ovsdb_field = Field::ovsdb("test", &Kind::Set(Box::new(Kind::Atomic(Atomic::String))));
        let expected_native = "struct Test {\n    test: Vec<String>,\n}\n";
        let expected_ovsdb = "struct Test {\n    test: ovsdb::protocol::Set<String>,\n}\n";

        assert_eq!(&test_struct(&native_field), expected_native);
        assert_eq!(&test_struct(&ovsdb_field), expected_ovsdb);
    }

    #[test]
    fn test_field_uuid_set() {
        let native_field = Field::native("test", &Kind::Set(Box::new(Kind::Atomic(Atomic::Uuid))));
        let ovsdb_field = Field::ovsdb("test", &Kind::Set(Box::new(Kind::Atomic(Atomic::Uuid))));
        let expected_native = "struct Test {\n    test: Vec<ovsdb::protocol::Uuid>,\n}\n";
        let expected_ovsdb = "struct Test {\n    test: ovsdb::protocol::UuidSet,\n}\n";

        assert_eq!(&test_struct(&native_field), expected_native);
        assert_eq!(&test_struct(&ovsdb_field), expected_ovsdb);
    }
}
