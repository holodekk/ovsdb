use std::convert::From;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::schema::Column;

use super::{Attribute, FieldEnum};

#[derive(Debug)]
pub struct Field {
    name: syn::Ident,
    attributes: Vec<Attribute>,
    kind: TokenStream,
    enumerations: Vec<FieldEnum>,
}

impl Field {
    pub fn new<S>(name: S) -> Self
    where
        S: AsRef<str>,
    {
        let mut attributes: Vec<Attribute> = vec![];
        let sanitized = Self::sanitize(&name, &mut attributes);
        Self {
            name: format_ident!("{}", sanitized),
            attributes,
            kind: quote! { INVALID },
            enumerations: vec![],
        }
    }

    fn sanitize<S>(name: S, attrs: &mut Vec<Attribute>) -> String
    where
        S: AsRef<str>,
    {
        match name.as_ref() {
            "type" => {
                attrs.push(Attribute::from("#[serde(rename = \"type\")]"));
                "kind".to_string()
            }
            _ => name.as_ref().into(),
        }
    }

    pub fn set_kind(&mut self, kind: TokenStream) {
        self.kind = kind;
    }

    pub fn add_attribute<S>(&mut self, attr: S)
    where
        S: AsRef<str>,
    {
        self.attributes.push(Attribute::from(attr));
    }

    pub fn add_enumeration(&mut self, enumeration: FieldEnum) {
        self.enumerations.push(enumeration);
    }

    pub fn enumerations(&self) -> &Vec<FieldEnum> {
        &self.enumerations
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_ident = &self.name;
        let field_kind = &self.kind;
        let attrs = &self.attributes;

        tokens.extend(quote! {
            #(#attrs)*
            #field_ident: #field_kind
        });
    }
}

impl From<&Column> for Field {
    fn from(column: &Column) -> Self {
        let k = &column.kind;
        let kind = if column.is_set() {
            quote! { protocol::Set<#k> }
        } else {
            quote! { #k }
        };
        let mut field = Self::new(&column.name);
        field.set_kind(kind);

        if let crate::schema::Kind::String(c) = &column.kind {
            if let Some(options) = &c.options {
                let name = column.name.to_case(Case::UpperCamel);
                let enumeration = FieldEnum::builder()
                    .name(name)
                    .attribute("#[derive(Debug, Deserialize, PartialEq, Serialize)]")
                    .attribute("#[serde(rename_all = \"snake_case\")]")
                    .values(options)
                    .value("None")
                    .default_value("None")
                    .build();
                let e = &enumeration.name();
                field.set_kind(quote! { #e });
                field.add_attribute("#[serde(with = \"protocol::enumeration\")]");
                field.add_enumeration(enumeration);
            }
        }

        field
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::prelude::*;

    #[test]
    fn test_field() {
        let expected = r#"struct Test {
    #[serde(with = "protocol::enumeration")]
    color: Color,
}
"#;
        let mut field = Field::new("color");
        field.set_kind(quote! { Color });
        field.add_attribute(r#"#[serde(with = "protocol::enumeration")]"#);
        let mut buffer = Vec::new();
        let parsed: syn::File = syn::parse2(quote! {
        struct Test {
            #field
        }})
        .unwrap();
        buffer
            .write_all(prettyplease::unparse(&parsed).as_bytes())
            .unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), expected);
    }
}
