use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;

use ovsdb::schema::Table;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_quote;

use crate::{name_to_ident, str_to_name, Attributes, Enumeration, Field, Kind};

pub(crate) struct Entity<'a> {
    name: &'a str,
    native_fields: Vec<Field>,
    proxy_fields: Vec<Field>,
    enumerations: Vec<Enumeration>,
}

impl<'a> Entity<'a> {
    fn name(&self) -> &'a str {
        self.name
    }

    fn native_name(&self) -> String {
        str_to_name(self.name())
    }

    fn native_ident(&self) -> syn::Ident {
        name_to_ident(self.native_name())
    }

    fn proxy_name(&self) -> String {
        format!("{}Proxy", str_to_name(self.name))
    }

    fn proxy_ident(&self) -> syn::Ident {
        name_to_ident(self.proxy_name())
    }

    fn native_fields(&self) -> &Vec<Field> {
        &self.native_fields
    }

    fn proxy_fields(&self) -> &Vec<Field> {
        &self.proxy_fields
    }

    fn enumerations(&self) -> &Vec<Enumeration> {
        &self.enumerations
    }

    fn model(&self) -> syn::ItemStruct {
        Self::build_struct(
            &self.native_ident(),
            self.native_fields(),
            &Attributes::new(&[
                "#[derive(Clone, Debug, Deserialize, Serialize)]",
                &format!(
                    "#[serde(from = \"{proxy_name}\", into = \"{proxy_name}\")]",
                    proxy_name = &self.proxy_name()
                ),
            ]),
        )
    }

    fn model_to_proxy(&self) -> syn::ItemImpl {
        Self::build_conversion(
            &self.native_ident(),
            &self.proxy_ident(),
            &self
                .native_fields()
                .iter()
                .map(|f| {
                    let field_ident = f.ident();
                    let other_ident = name_to_ident("other");
                    if f.is_atomic() {
                        parse_quote! { #field_ident: #other_ident.#field_ident }
                    } else {
                        parse_quote! { #field_ident: #other_ident.#field_ident.into() }
                    }
                })
                .collect(),
        )
    }

    fn model_impl(&self) -> syn::ItemImpl {
        let name = self.name();
        let ident = self.native_ident();

        parse_quote! {
            impl Entity for #ident {
                fn table_name() -> &'static str {
                    #name
                }
            }
        }
    }

    fn proxy(&self) -> syn::ItemStruct {
        Self::build_struct(
            &self.proxy_ident(),
            self.proxy_fields(),
            &Attributes::new(&["#[derive(Debug, Deserialize, Serialize)]"]),
        )
    }

    fn proxy_to_model(&self) -> syn::ItemImpl {
        Self::build_conversion(
            &self.proxy_ident(),
            &self.native_ident(),
            &self
                .proxy_fields()
                .iter()
                .map(|f| {
                    let field_ident = f.ident();
                    let other_ident = name_to_ident("other");
                    if f.is_atomic() {
                        parse_quote! { #field_ident: #other_ident.#field_ident }
                    } else {
                        parse_quote! { #field_ident: #other_ident.#field_ident.into() }
                    }
                })
                .collect(),
        )
    }

    pub(crate) fn from_table(table: &'a Table) -> Self {
        let mut native_fields: Vec<Field> = vec![];
        let mut proxy_fields: Vec<Field> = vec![];
        let mut enumerations: Vec<Enumeration> = vec![];

        table.columns().iter().for_each(|c| {
            let kind = Kind::from_column(c);
            native_fields.push(Field::native(c.name(), &kind));
            proxy_fields.push(Field::ovsdb(c.name(), &kind));

            if let Some(choices) = c.kind().key().choices().as_ref() {
                enumerations.push(Enumeration::builder()
                    .name(c.name())
                    .attribute(
                        "#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]",
                    )
                    .values((*choices).deref())
                    .build());
            }
        });

        Self {
            name: table.name(),
            native_fields,
            proxy_fields,
            enumerations,
        }
    }

    pub(crate) fn to_file<P>(&self, filename: P) -> super::Result<()>
    where
        P: AsRef<Path>,
    {
        let mut output_file = File::create(filename)?;
        let parsed: syn::File = parse_quote! { #self };
        output_file.write_all(prettyplease::unparse(&parsed).as_bytes())?;
        Ok(())
    }

    fn build_struct(
        ident: &syn::Ident,
        fields: &Vec<Field>,
        attributes: &Attributes,
    ) -> syn::ItemStruct {
        parse_quote! {
            #(#attributes)*
            pub struct #ident {
                #(#fields),*
            }
        }
    }

    fn build_conversion(
        ident: &syn::Ident,
        other: &syn::Ident,
        fields: &Vec<syn::FieldValue>,
    ) -> syn::ItemImpl {
        parse_quote! {
            impl From<#other> for #ident {
                fn from(other: #other) -> Self {
                    Self {
                        #(#fields),*
                    }
                }
            }
        }
    }
}

impl<'a> ToTokens for Entity<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let enumerations = self.enumerations();
        let model = self.model();
        let model_impl = self.model_impl();
        let proxy = self.proxy();
        let model_to_proxy = self.model_to_proxy();
        let proxy_to_model = self.proxy_to_model();
        tokens.extend(quote! {
            use serde::{Deserialize, Serialize};
            use ovsdb::Entity;

            #(#enumerations)*
            #model
            #model_impl
            #proxy
            #model_to_proxy
            #proxy_to_model
        });
    }
}
