use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use convert_case::{Case, Casing};
use quote::ToTokens;

use crate::schema::Schema;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
    #[error("Token parse error")]
    Tokens(#[from] syn::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// Example usage (build.rs)
//
// use std::path::Path;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let schema = ovsdb::schema::Schema::from_file(Path::new("vswitch.ovsschema"))?;

//     ovsdb::generator::generate_api(&schema, Path::new("src/vswitch"))?;

//     Ok(())
// }

pub fn generate_api(schema: &Schema, directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory)?;

    let mod_filename = directory.join("mod.rs");
    let mut mod_file = File::create(mod_filename)?;
    for t in &schema.tables {
        let filename = directory.join(format!("{}.rs", t.name.to_case(Case::Snake)));
        let mut output_file = File::create(filename)?;

        let parsed: syn::File = syn::parse2(t.to_token_stream())?;
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
