use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use convert_case::{Case, Casing};
use quote::quote;

use crate::schema::{Column, Schema, Table};

mod model;
pub use model::*;
mod field;
pub use field::*;
mod field_enum;
pub use field_enum::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
    #[error("Token parse error")]
    Tokens(#[from] syn::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn generate_attribute<S>(attr: S) -> Vec<syn::Attribute>
where
    S: AsRef<str>,
{
    syn::parse_str::<syn::DeriveInput>(&format!("{}\nstruct fake;", attr.as_ref()))
        .unwrap()
        .attrs
}

pub fn generate_attributes<T, S>(attrs: T) -> Vec<syn::Attribute>
where
    T: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    attrs
        .into_iter()
        .map(generate_attribute)
        .flatten()
        .collect()
}

pub fn generate_enum(table: &Table, column: &Column, options: &Vec<String>) -> Result<FieldEnum> {
    let enum_name = format!(
        "{}{}",
        table.name.to_case(Case::UpperCamel),
        column.name.to_case(Case::UpperCamel)
    );
    let enum_builder = FieldEnum::build(&enum_name);
    enum_builder.attribute("#[derive(Debug, Deserialize, PartialEq, Serialize)]");
    enum_builder.attribute("#[serde(rename_all = \"snake_case\")]");
    for o in options {
        enum_builder.value(o);
    }
    enum_builder.value("None");
    enum_builder.default("None");
    Ok(enum_builder.build())
}

pub fn generate_model(table: &Table) -> Result<Model> {
    let model_builder = Model::build(&table.name);
    model_builder.attribute("#[derive(Debug, Deserialize, Serialize)]");

    for column in &table.columns {
        let kind = &column.kind;
        let field_builder = Field::build(&column.name);
        if column.is_set() {
            field_builder.kind(quote! { protocol::Set<#kind> });
        } else {
            field_builder.kind(quote! { #kind });
        }

        if let crate::schema::DataType::String(c) = &column.kind {
            if let Some(options) = &c.options {
                let enumeration = generate_enum(table, column, options)?;
                let e = &enumeration.name;
                field_builder.kind(quote! { #e });
                model_builder.enumeration(enumeration);
                field_builder.attribute("#[serde(with = \"protocol::enumeration\")]");
            }
        }
        model_builder.field(field_builder.build());
    }

    Ok(model_builder.build())
}

pub fn generate_models(schema: &Schema, directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory)?;

    let mod_filename = directory.join("mod.rs");
    let mut mod_file = File::create(mod_filename)?;
    for t in &schema.tables {
        let filename = directory.join(format!("{}.rs", t.name.to_case(Case::Snake)));
        let mut output_file = File::create(filename)?;
        let model = generate_model(t)?;

        let tokens = quote! {
            use serde::{Deserialize, Serialize};
            use ovsdb::{protocol, client};
            #model
        };

        let parsed: syn::File = syn::parse2(tokens)?;
        output_file.write_all(prettyplease::unparse(&parsed).as_bytes())?;

        mod_file.write_all(
            format!(
                "mod {};\npub use {}::*;\n",
                t.name.to_case(Case::Snake),
                t.name.to_case(Case::Snake)
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}
