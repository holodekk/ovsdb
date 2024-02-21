//! `ovsdb-build` compiles OVSDB schema objects into rust entity definitions and proxies for use with
//! `ovsdb`
//!
//! # Dependencies
//!
//! ```toml
//! [dependencies]
//! ovsdb = { version = <ovsdb-version>, features = ["client"] }
//! serde = { version = <serde-version>, features = ["derive"] }
//!
//! [build-dependencies]
//! ovsdb-build = <ovsdb-version>
//!
//! # Examples
//!
//! ```rust,no_run
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     ovsdb_build::configure().compile("/path/to/vswitch.ovsschema", "vswitch");
//!     Ok(())
//! }
//! ```

// Built-in Lints
#![warn(
    unreachable_pub,
    missing_debug_implementations,
    missing_copy_implementations,
    elided_lifetimes_in_paths,
    missing_docs
)]
// Clippy lints
#![allow(
    clippy::match_same_arms,
    clippy::needless_doctest_main,
    clippy::map_unwrap_or,
    clippy::redundant_field_names,
    clippy::type_complexity
)]
#![warn(
    clippy::unwrap_used,
    // clippy::print_stdout,
    clippy::mut_mut,
    clippy::non_ascii_literal,
    clippy::similar_names,
    clippy::unicode_not_nfc,
    clippy::enum_glob_use,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::used_underscore_binding
)]
#![deny(unsafe_code)]

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

/// Error type for Schema and generation errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// General IO error
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
    /// Invalid code generation
    #[error("Token parse error")]
    Tokens(#[from] syn::Error),
    /// OVSDB parsing error
    #[error("Parsing error")]
    OVSDB(#[from] ovsdb::Error),
}

/// Standard result for all build related methods.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn str_to_name<T>(str: T) -> String
where
    T: AsRef<str>,
{
    str.as_ref().to_case(Case::UpperCamel)
}

pub(crate) fn name_to_ident<T>(name: T) -> syn::Ident
where
    T: AsRef<str>,
{
    format_ident!("{}", name.as_ref())
}

/// Schema entity builder
#[derive(Clone, Debug, Default)]
pub struct Builder {
    out_dir: Option<PathBuf>,
}

impl Builder {
    fn new() -> Self {
        Self::default()
    }

    fn generate_modules(&self, schema: &Schema, directory: &Path) -> Result<()> {
        std::fs::create_dir_all(directory)?;

        let mod_filename = directory.join("mod.rs");
        let mut mod_file = File::create(mod_filename)?;
        for table in schema.tables() {
            let filename = directory.join(format!("{}.rs", table.name().to_case(Case::Snake)));
            let entity = Entity::from_table(table);
            entity.to_file(&filename)?;

            mod_file.write_all(
                format!(
                    "mod {table_name};\npub use {table_name}::*;\n",
                    table_name = &table.name().to_case(Case::Snake)
                )
                .as_bytes(),
            )?;
        }
        Ok(())
    }

    /// Compile the `.ovsschema` file into rust objects.
    pub fn compile<P>(self, schema_file: P, module: P) -> Result<()>
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

        let schema = ovsdb::schema::Schema::from_file(schema_file)?;

        output_dir.push(module);

        self.generate_modules(&schema, &output_dir)
    }
}

/// Configure `ovsdb-build` code generation options.
pub fn configure() -> Builder {
    Builder::new()
}
