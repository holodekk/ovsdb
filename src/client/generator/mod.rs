use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use convert_case::{Case, Casing};
use quote::quote;

use crate::schema::Schema;

mod attribute;
mod field;
mod field_enum;
mod model;

use attribute::Attribute;
use field::Field;
use field_enum::FieldEnum;
use model::Model;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
    #[error("Token parse error")]
    Tokens(#[from] syn::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn generate_models(schema: &Schema, directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory)?;

    let mod_filename = directory.join("mod.rs");
    let mut mod_file = File::create(mod_filename)?;
    for table in &schema.tables {
        let filename = directory.join(format!("{}.rs", table.name.to_case(Case::Snake)));
        let mut output_file = File::create(filename)?;

        let model = Model::builder()
            .name(&table.name)
            .attribute("#[derive(Debug, Deserialize, Serialize)]")
            .fields(table.columns.iter().map(|c| c.into()).collect())
            .build();

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
                table.name.to_case(Case::Snake),
                table.name.to_case(Case::Snake)
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}
