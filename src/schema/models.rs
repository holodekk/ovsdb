use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::schema::{
    column::{Column, DataType},
    table::Table,
    Schema,
};

// Example usage (build.rs)
//
// use std::path::Path;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let schema = ovsdb_schema::load(Path::new("vswitch.ovsschema"))?;

//     ovsdb_schema::output_client(&schema, Path::new("src/vswitch/mod.rs"));

//     Ok(())
// }

pub fn data_type_to_ident(dt: &DataType) -> Ident {
    match dt {
        DataType::Boolean => format_ident!("{}", "bool"),
        DataType::Integer(_) => format_ident!("{}", "i64"),
        DataType::Real(_) => format_ident!("{}", "f64"),
        DataType::String(_) => format_ident!("{}", "String"),
        DataType::Uuid { .. } => format_ident!("{}", "Uuid"),
        DataType::Map { .. } => format_ident!("{}", "HashMap"),
        _ => panic!("Invalid data type"),
    }
}

pub fn dump_column(c: &Column) -> TokenStream {
    let column_ident = match c.name.as_str() {
        "type" => format_ident!("{}", "kind"),
        _ => format_ident!("{}", &c.name),
    };
    let type_ident = data_type_to_ident(&c.kind);

    match &c.kind {
        DataType::Uuid { .. } => {
            let uuid_module = format_ident!("{}", "uuid");
            let uuid_type = format_ident!("{}", "Uuid");
            quote! {
                pub #column_ident: #uuid_module::#uuid_type
            }
        }
        DataType::Map { key, value } => {
            let key_ident = data_type_to_ident(key);
            let val_ident = data_type_to_ident(value);

            quote! {
                pub #column_ident: #type_ident<#key_ident, #val_ident>
            }
        }
        _ => quote! { pub #column_ident: #type_ident },
    }
}

pub fn generate_model(t: &Table) -> Model {
    let table_name = format_ident!("{}", t.name.to_case(Case::UpperCamel));
    let table_name_fn = format_ident!("{}", "table_name");
    let table_name_str = t.name.clone();
    let members: Vec<TokenStream> = t.columns.iter().map(dump_column).collect();

    let model_def = quote! {
        #[derive(Debug, Deserialize)]
        pub struct #table_name {
          #(#members,)*
        }
    };

    let model_impl = quote! {
        impl Entity for #table_name {
          fn #table_name_fn() -> &'static str {
            #table_name_str
          }
        }
    };

    Model {
        model_def,
        model_impl,
    }
}

pub fn generate_use_declarations(_s: &Schema) -> TokenStream {
    quote! {
        use std::collections::HashMap;

        use serde::Deserialize;
        use uuid::Uuid;

        use crate::Entity;
    }
}

pub fn generate_consts(s: &Schema) -> TokenStream {
    let db_name_str = s.name.clone();

    quote! {
        pub const DATABASE_NAME: &str = #db_name_str;
    }
}

pub struct Model {
    pub model_def: TokenStream,
    pub model_impl: TokenStream,
}

pub struct Client {
    pub use_declarations: TokenStream,
    pub consts: TokenStream,
    pub models: Vec<Model>,
}

pub fn generate_models(s: &Schema) -> Vec<Model> {
    s.tables.iter().map(Model::generate_model).collect()
}

pub fn generate_client(s: &Schema) -> Client {
    let use_declarations = generate_use_declarations(s);
    let consts = generate_consts(s);
    let models = generate_models(s);

    Client {
        use_declarations,
        consts,
        models,
    }
}

pub fn output_tokens(output_file: &mut File, tokens: &TokenStream) -> Result<(), std::io::Error> {
    let parsed: syn::File = syn::parse2(tokens.clone())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "foo"))?;
    let o = prettyplease::unparse(&parsed);
    output_file.write_all(o.as_bytes())?;
    output_file.write_all(b"\n")
}

pub fn output_client(schema: &Schema, filename: &Path) -> Result<(), std::io::Error> {
    let mut output_file = File::create(filename)?;

    let client = generate_client(schema);

    output_tokens(&mut output_file, &client.use_declarations)?;
    output_file.write_all(b"\n")?;

    output_tokens(&mut output_file, &client.consts)?;
    output_file.write_all(b"\n")?;

    client
        .models
        .iter()
        .map(|m: &Model| -> Result<(), std::io::Error> {
            output_tokens(&mut output_file, &m.model_def)?;
            output_tokens(&mut output_file, &m.model_impl)
        })
        .collect()
}
