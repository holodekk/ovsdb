use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{Attribute, Field, FieldEnum};

#[derive(Debug)]
pub struct Model {
    name: syn::Ident,
    attributes: Vec<Attribute>,
    fields: Vec<Field>,
}

impl Model {
    pub fn builder() -> ModelBuilder {
        ModelBuilder::new()
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            name: format_ident!("{}", "INVALID"),
            attributes: vec![],
            fields: vec![],
        }
    }
}

impl ToTokens for Model {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let table_name = self.name.to_string();
        let struct_ident = &self.name;
        let attrs = &self.attributes;
        let fields = &self.fields;
        let enums: Vec<&FieldEnum> = fields.iter().flat_map(|f| f.enumerations()).collect();

        tokens.extend(quote! {
            #(#enums)*
            #(#attrs)*
            pub struct #struct_ident {
                #(#fields),*
            }
            impl Entity for #struct_ident {
                fn table_name() -> &'static str {
                    #table_name
                }
            }
        })
    }
}

#[derive(Debug, Default)]
pub struct ModelBuilder {
    target: Model,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name<S>(mut self, name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.target.name = format_ident!("{}", name.as_ref().to_case(Case::UpperCamel));
        self
    }

    pub fn attribute<S>(mut self, attr: S) -> Self
    where
        S: AsRef<str>,
    {
        self.target.attributes.push(Attribute::from(attr));
        self
    }

    pub fn fields(mut self, mut fields: Vec<Field>) -> Self {
        self.target.fields.append(&mut fields);
        self
    }

    pub fn build(self) -> Model {
        self.target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::prelude::*;

    #[test]
    fn test_model() {
        let expected = r#"pub enum TestColor {
    Red,
    Blue,
    Green,
    None,
}
impl Default for TestColor {
    fn default() -> Self {
        Self::None
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Test {
    #[serde(deserialize_with = "deserialize_enum")]
    color: TestColor,
}
impl Entity for Test {
    fn table_name() -> &'static str {
        "Test"
    }
}
"#;
        let field_enum = FieldEnum::builder()
            .name("TestColor")
            .value("red")
            .value("blue")
            .value("green")
            .value("none")
            .default_value("none")
            .build();

        let mut field = Field::new("color");
        field.set_kind(quote! { TestColor });
        field.add_attribute("#[serde(deserialize_with = \"deserialize_enum\")]");
        field.add_enumeration(field_enum);

        let model = Model::builder()
            .name("test")
            .attribute("#[derive(Debug, Deserialize, Serialize)]")
            .fields(vec![field])
            .build();

        let mut buffer = Vec::new();
        let parsed: syn::File = syn::parse2(quote! { #model }).unwrap();
        buffer
            .write_all(prettyplease::unparse(&parsed).as_bytes())
            .unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), expected);
    }
}
