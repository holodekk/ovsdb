use std::cell::RefCell;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::Attribute;

#[derive(Debug)]
pub struct FieldEnum {
    name: syn::Ident,
    attributes: Vec<Attribute>,
    values: Vec<syn::Ident>,
    default: Option<String>,
}

impl FieldEnum {
    pub fn name(&self) -> &syn::Ident {
        &self.name
    }

    pub fn builder() -> FieldEnumBuilder {
        FieldEnumBuilder::new()
    }
}

impl Default for FieldEnum {
    fn default() -> Self {
        Self {
            name: format_ident!("{}", "INVALID"),
            attributes: vec![],
            values: vec![],
            default: None,
        }
    }
}

impl ToTokens for FieldEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let enum_ident = &self.name;
        let attrs = &self.attributes;
        let values = &self.values;

        tokens.extend(quote! {
            #(#attrs)*
            pub enum #enum_ident {
                #(#values),*
            }
        });

        if let Some(def) = &self.default {
            let default_ident = format_ident!("{}", def.to_case(Case::UpperCamel));
            tokens.extend(quote! {
                impl Default for #enum_ident {
                    fn default() -> Self {
                        Self::#default_ident
                    }
                }
            });
        }
    }
}

pub struct FieldEnumBuilder {
    target: RefCell<FieldEnum>,
}

impl FieldEnumBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name<S>(self, name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.target.borrow_mut().name =
            format_ident!("{}", name.as_ref().to_case(Case::UpperCamel));
        self
    }

    pub fn attribute<S>(self, attr: S) -> Self
    where
        S: AsRef<str>,
    {
        self.target
            .borrow_mut()
            .attributes
            .push(Attribute::from(attr));
        self
    }

    pub fn value<S>(self, value: S) -> Self
    where
        S: AsRef<str>,
    {
        self.target.borrow_mut().values.push(format_ident!(
            "{}",
            value.as_ref().to_case(Case::UpperCamel)
        ));
        self
    }

    pub fn values<T, S>(self, values: T) -> Self
    where
        T: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for value in values {
            self.target.borrow_mut().values.push(format_ident!(
                "{}",
                value.as_ref().to_case(Case::UpperCamel)
            ));
        }
        self
    }

    pub fn default_value<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        self.target.borrow_mut().default = Some(value.into());
        self
    }

    pub fn build(self) -> FieldEnum {
        self.target.into_inner()
    }
}

impl Default for FieldEnumBuilder {
    fn default() -> Self {
        Self {
            target: RefCell::new(FieldEnum::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::prelude::*;

    #[test]
    fn test_enum() {
        let expected = r#"#[derive(Deserialize, Serialize)]
pub enum Test {
    Blue,
    Red,
    Green,
    None,
}
impl Default for Test {
    fn default() -> Self {
        Self::None
    }
}
"#;
        let value = FieldEnum::builder()
            .name("test")
            .attribute("#[derive(Deserialize, Serialize)]")
            .value("blue")
            .value("red")
            .value("green")
            .value("none")
            .default_value("None")
            .build();
        let mut buffer = Vec::new();
        let parsed: syn::File = syn::parse2(quote! { #value }).unwrap();
        buffer
            .write_all(prettyplease::unparse(&parsed).as_bytes())
            .unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), expected);
    }
}
