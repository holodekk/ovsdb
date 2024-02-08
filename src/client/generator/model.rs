use std::cell::RefCell;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{Field, FieldEnum};

pub struct Model {
    name: syn::Ident,
    attributes: Vec<syn::Attribute>,
    fields: Vec<Field>,
    enums: Vec<FieldEnum>,
}

impl Model {
    pub fn new<S>(name: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            name: format_ident!("{}", name.as_ref().to_case(Case::UpperCamel)),
            attributes: vec![],
            fields: vec![],
            enums: vec![],
        }
    }

    pub fn build<S>(name: S) -> ModelBuilder
    where
        S: AsRef<str>,
    {
        ModelBuilder::new(Self::new(name))
    }
}

impl ToTokens for Model {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let table_name = self.name.to_string();
        let struct_ident = &self.name;
        let attrs = &self.attributes;
        let enums = &self.enums;
        let fields = &self.fields;
        tokens.extend(quote! {
            #(#enums)*
            #(#attrs)*
            pub struct #struct_ident {
                #(#fields),*
            }
            impl client::Entity for #struct_ident {
                fn table_name() -> &'static str {
                    #table_name
                }
            }
        })
    }
}

pub struct ModelBuilder {
    target: RefCell<Model>,
}

impl ModelBuilder {
    pub fn new(target: Model) -> Self {
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

    pub fn field(&self, field: Field) {
        self.target.borrow_mut().fields.push(field);
    }

    pub fn enumeration(&self, enumeration: FieldEnum) {
        self.target.borrow_mut().enums.push(enumeration);
    }

    pub fn build(self) -> Model {
        self.target.into_inner()
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
impl client::Entity for Test {
    fn table_name() -> &'static str {
        "Test"
    }
}
"#;
        let enum_builder = FieldEnum::build("TestColor");
        enum_builder.value("red");
        enum_builder.value("blue");
        enum_builder.value("green");
        enum_builder.value("none");
        enum_builder.default("none");

        let field_builder = Field::build("color");
        field_builder.attribute("#[serde(deserialize_with = \"deserialize_enum\")]");
        field_builder.kind(quote! { TestColor });

        let model_builder = Model::build("test");
        model_builder.attribute("#[derive(Debug, Deserialize, Serialize)]");
        model_builder.enumeration(enum_builder.build());
        model_builder.field(field_builder.build());

        let model = model_builder.build();
        let mut buffer = Vec::new();
        let parsed: syn::File = syn::parse2(quote! { #model }).unwrap();
        buffer
            .write_all(prettyplease::unparse(&parsed).as_bytes())
            .unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), expected);
    }
}
