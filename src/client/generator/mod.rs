use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use convert_case::{Case, Casing};
use quote::{format_ident, quote};

use crate::schema::Schema;

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

// Example usage (build.rs)
//
// use std::path::Path;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let schema = ovsdb::schema::Schema::from_file(Path::new("vswitch.ovsschema"))?;

//     ovsdb::generator::generate_api(&schema, Path::new("src/vswitch"))?;

//     Ok(())
// }

pub fn generate_models(schema: &Schema, directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory)?;

    let mod_filename = directory.join("mod.rs");
    let mut mod_file = File::create(mod_filename)?;
    for t in &schema.tables {
        let filename = directory.join(format!("{}.rs", t.name.to_case(Case::Snake)));
        let mut output_file = File::create(filename)?;
        let model_builder = Model::build(&t.name);
        model_builder.attribute("#[derive(Debug, Deserialize, Serialize)]");

        for column in &t.columns {
            let field_builder = Field::build(&column.name);

            let kind = &column.kind;
            // let kind = quote! { protocol::Map<String, String> };
            field_builder.kind(quote! { #kind });

            if let crate::schema::DataType::String(c) = &column.kind {
                if let Some(options) = &c.options {
                    let enum_name = format!(
                        "{}{}",
                        t.name.to_case(Case::UpperCamel),
                        column.name.to_case(Case::UpperCamel)
                    );
                    let enum_builder = FieldEnum::build(&enum_name);
                    enum_builder.attribute("#[derive(Debug, Deserialize, PartialEq, Serialize)]");
                    for o in options {
                        enum_builder.value(o);
                    }
                    enum_builder.value("None");
                    enum_builder.default("None");

                    model_builder.enumeration(enum_builder.build());
                    field_builder
                        .attribute("#[serde(deserialize_with = \"protocol::deserialize_enum\")]");
                    let e = format_ident!("{}", enum_name);
                    field_builder.kind(quote! { #e });
                }
            }
            model_builder.field(field_builder.build());
        }

        let model = model_builder.build();

        let tokens = quote! {
            use serde::{Deserialize, Serialize};

            use ovsdb::{protocol, Entity};
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
