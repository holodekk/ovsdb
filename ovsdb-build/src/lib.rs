use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use convert_case::{Case, Casing};
use ovsdb::schema::Schema;
use quote::format_ident;

mod attributes;
mod entity;
mod enumeration;
mod field;
use attributes::Attributes;
use entity::Entity;
use enumeration::Enumeration;
use field::{Field, Kind};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
    #[error("Token parse error")]
    Tokens(#[from] syn::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn str_to_name<T>(str: T) -> String
where
    T: AsRef<str>,
{
    str.as_ref().to_case(Case::UpperCamel)
}

pub fn name_to_ident<T>(name: T) -> syn::Ident
where
    T: AsRef<str>,
{
    format_ident!("{}", name.as_ref())
}

#[derive(Clone, Debug, Default)]
pub struct Builder {
    out_dir: Option<PathBuf>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_modules(&self, schema: &Schema, directory: &Path) -> Result<()> {
        std::fs::create_dir_all(directory)?;

        let mod_filename = directory.join("mod.rs");
        let mut mod_file = File::create(mod_filename)?;
        for table in &schema.tables {
            let filename = directory.join(format!("{}.rs", table.name.to_case(Case::Snake)));
            let entity = Entity::from_table(table);
            entity.to_file(&filename)?;

            mod_file.write_all(
                format!(
                    "mod {table_name};\npub use {table_name}::*;\n",
                    table_name = &table.name.to_case(Case::Snake)
                )
                .as_bytes(),
            )?;
        }
        Ok(())
    }

    pub fn compile<P>(self, schema_file: P, module: P)
    where
        P: AsRef<Path>,
    {
        let mut output_dir = match &self.out_dir {
            Some(dir) => dir.to_path_buf(),
            None => match std::env::var("OUT_DIR") {
                Ok(val) => PathBuf::from(val),
                Err(_) => todo!(),
            },
        };

        let schema = ovsdb::schema::Schema::from_file(schema_file).unwrap();

        output_dir.push(module);

        self.generate_modules(&schema, &output_dir).unwrap();
    }
}

pub fn configure() -> Builder {
    Builder::new()
}
