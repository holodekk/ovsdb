use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use super::column::Column;

#[derive(Debug, Deserialize)]
pub struct Table {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "isRoot", default)]
    pub is_root: Option<bool>,
    #[serde(rename = "maxRows", default)]
    pub max_rows: Option<i64>,
    #[serde(deserialize_with = "deserialize_columns")]
    pub columns: Vec<Column>,
}

impl ToTokens for Table {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let def = TableDef(self).to_token_stream();
        let imp = TableImpl(self).to_token_stream();

        tokens.extend(quote! {
            use serde::{Deserialize, Serialize};

            use ovsdb::{protocol, Entity};

            #def
            #imp
        });
    }
}

struct TableDef<'a>(pub &'a Table);

impl<'a> ToTokens for TableDef<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let column_tokens: Vec<TokenStream> =
            self.0.columns.iter().map(|c| c.to_token_stream()).collect();
        let table_name = format_ident!("{}", self.0.name.to_case(Case::UpperCamel));
        let table_tokens = quote! {
            #[derive(Debug, Deserialize, Serialize)]
            pub struct #table_name {
                #( #column_tokens),*
            }
        };
        tokens.extend(table_tokens);
    }
}

impl<'a> std::convert::TryFrom<TableDef<'a>> for syn::ItemStruct {
    type Error = syn::Error;

    fn try_from(def: TableDef<'a>) -> Result<Self, Self::Error> {
        let tokens = def.to_token_stream();
        let parsed: syn::ItemStruct = syn::parse2(tokens)?;
        Ok(parsed)
    }
}

struct TableImpl<'a>(pub &'a Table);

impl<'a> ToTokens for TableImpl<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let table_ident = format_ident!("{}", self.0.name.to_case(Case::UpperCamel));
        let table = self.0.name.to_string();
        tokens.extend(quote! {
            impl Entity for #table_ident {
              fn table_name() -> &'static str {
                #table
              }
            }
        });
    }
}

impl<'a> std::convert::TryFrom<TableImpl<'a>> for syn::ItemImpl {
    type Error = syn::Error;

    fn try_from(imp: TableImpl<'a>) -> Result<Self, Self::Error> {
        let tokens = imp.to_token_stream();
        let parsed: Self = syn::parse2(tokens)?;
        Ok(parsed)
    }
}

fn deserialize_columns<'de, D>(de: D) -> Result<Vec<Column>, D::Error>
where
    D: Deserializer<'de>,
{
    let columns = Value::deserialize(de)?
        .as_object()
        .expect("convert table `columns` to json object")
        .iter()
        .map(|(k, v)| {
            let mut c: Column = Column::deserialize(v).unwrap();
            c.name = k.to_string();
            c
        })
        .collect();
    Ok(columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_table() {
        let data = r#"{ "columns": { "name": { "type": "string", "mutable": false } }, "isRoot": false, "maxRows": 100 }"#;
        let t: Table = serde_json::from_str(data).unwrap();
        assert_eq!(t.columns.len(), 1);
        assert_eq!(t.is_root, Some(false));
        assert_eq!(t.max_rows, Some(100));
    }
}
