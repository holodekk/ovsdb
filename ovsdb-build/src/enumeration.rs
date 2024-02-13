use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{name_to_ident, str_to_name, Attributes};

#[derive(Clone, Debug)]
struct EnumerationValue {
    attributes: Attributes,
    ident: syn::Ident,
}

impl EnumerationValue {
    fn from_str<T>(str: T) -> Self
    where
        T: AsRef<str>,
    {
        let ident = name_to_ident(str_to_name(&str));
        Self {
            ident,
            attributes: Attributes::default(),
        }
    }

    fn add_attribute<T>(&mut self, attr: T)
    where
        T: AsRef<str>,
    {
        self.attributes.add(&attr);
    }

    fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl ToTokens for EnumerationValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident();
        let attrs = self.attributes();

        tokens.extend(quote! {
            #(#attrs)*
            #ident
        });
    }
}

#[derive(Debug)]
pub(crate) struct Enumeration {
    ident: syn::Ident,
    attributes: Attributes,
    values: Vec<EnumerationValue>,
}

impl Enumeration {
    fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn values(&self) -> &Vec<EnumerationValue> {
        &self.values
    }

    pub(crate) fn builder<'a>() -> EnumerationBuilder<'a> {
        EnumerationBuilder::new()
    }
}

impl ToTokens for Enumeration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident();
        let attrs = self.attributes();
        let values = self.values();

        tokens.extend(quote! {
            #(#attrs)*
            pub enum #ident {
                #(#values),*
            }
        });
    }
}

#[derive(Debug)]
pub(crate) struct EnumerationBuilder<'a> {
    name: Option<&'a str>,
    attributes: Attributes,
    values: Vec<EnumerationValue>,
}

impl<'a> EnumerationBuilder<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            attributes: Attributes::default(),
            values: vec![],
        }
    }

    pub fn name(&mut self, name: &'a str) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn attribute<T>(&mut self, attr: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.attributes.add(&attr);
        self
    }

    pub fn value<S>(&mut self, value: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        let camelized = str_to_name(&value);
        let mut e = EnumerationValue::from_str(&camelized);

        if camelized != value.as_ref() {
            e.add_attribute(&format!("#[serde(rename = \"{}\")]", value.as_ref()));
        }
        self.values.push(e);

        self
    }

    pub fn values<T, S>(&mut self, values: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for value in values {
            self.value(&value);
        }
        self
    }

    pub fn build(&self) -> Enumeration {
        Enumeration {
            ident: name_to_ident(str_to_name(self.name.unwrap())),
            attributes: self.attributes.clone(),
            values: self.values.clone(),
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
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "green")]
    Green,
}
"#;
        let value = Enumeration::builder()
            .name("test")
            .attribute("#[derive(Deserialize, Serialize)]")
            .value("blue")
            .value("red")
            .value("green")
            .build();
        let mut buffer = Vec::new();
        let parsed: syn::File = syn::parse2(quote! { #value }).unwrap();
        buffer
            .write_all(prettyplease::unparse(&parsed).as_bytes())
            .unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), expected);
    }
}
