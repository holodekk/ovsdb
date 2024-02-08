use std::cell::RefCell;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

#[derive(Debug)]
pub struct Field {
    name: syn::Ident,
    attributes: Vec<syn::Attribute>,
    kind: TokenStream,
}

impl Field {
    pub fn new<S>(name: S) -> Self
    where
        S: AsRef<str>,
    {
        let mut attributes: Vec<syn::Attribute> = vec![];
        let sanitized = Self::sanitize(&name, &mut attributes);
        Self {
            name: format_ident!("{}", sanitized),
            attributes,
            kind: quote! { INVALID },
        }
    }

    fn sanitize<S>(name: S, attrs: &mut Vec<syn::Attribute>) -> String
    where
        S: AsRef<str>,
    {
        match name.as_ref() {
            "type" => {
                let mut attr = super::generate_attribute("#[serde(rename = \"type\")]");
                attrs.append(&mut attr);
                "kind".to_string()
            }
            _ => name.as_ref().into(),
        }
    }

    pub fn build<S>(name: S) -> FieldBuilder
    where
        S: AsRef<str>,
    {
        FieldBuilder::new(Self::new(name))
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

pub struct FieldBuilder {
    target: RefCell<Field>,
}

impl FieldBuilder {
    pub fn new(target: Field) -> Self {
        Self {
            target: RefCell::new(target),
        }
    }

    pub fn attribute<S>(&self, attr: S)
    where
        S: AsRef<str>,
    {
        let mut attrs = super::generate_attribute(attr);
        self.target.borrow_mut().attributes.append(&mut attrs);
    }

    pub fn kind(&self, kind: TokenStream) {
        self.target.borrow_mut().kind = kind;
    }

    pub fn build(self) -> Field {
        self.target.into_inner()
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
        let field_builder = Field::build("color");
        field_builder.attribute(r#"#[serde(with = "protocol::enumeration")]"#);
        field_builder.kind(quote! { Color });
        let field = field_builder.build();
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
