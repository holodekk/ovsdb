use std::cell::RefCell;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

pub struct FieldEnum {
    name: syn::Ident,
    attributes: Vec<syn::Attribute>,
    values: Vec<syn::Ident>,
    default: Option<String>,
}

impl FieldEnum {
    pub fn new<S>(name: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            name: format_ident!("{}", name.as_ref().to_case(Case::UpperCamel)),
            attributes: vec![],
            values: vec![],
            default: None,
        }
    }

    pub fn build<S>(name: S) -> FieldEnumBuilder
    where
        S: AsRef<str>,
    {
        FieldEnumBuilder::new(Self::new(name))
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
    pub fn new(target: FieldEnum) -> Self {
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

    pub fn value<S>(&self, value: S)
    where
        S: AsRef<str>,
    {
        self.target.borrow_mut().values.push(format_ident!(
            "{}",
            value.as_ref().to_case(Case::UpperCamel)
        ));
    }

    pub fn default<S>(&self, value: S)
    where
        S: Into<String>,
    {
        self.target.borrow_mut().default = Some(value.into());
    }

    pub fn build(self) -> FieldEnum {
        self.target.into_inner()
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
        let builder = FieldEnum::build("test");
        builder.attribute("#[derive(Deserialize, Serialize)]");
        builder.value("blue");
        builder.value("red");
        builder.value("green");
        builder.value("none");
        builder.default("None");
        let model_enum = builder.build();
        let mut buffer = Vec::new();
        let parsed: syn::File = syn::parse2(quote! { #model_enum }).unwrap();
        buffer
            .write_all(prettyplease::unparse(&parsed).as_bytes())
            .unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), expected);
    }
}
