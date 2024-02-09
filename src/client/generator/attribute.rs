use std::convert::From;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug)]
pub struct Attribute {
    value: Vec<syn::Attribute>,
}

impl<T> From<T> for Attribute
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let template = format!("{}\nstruct dummy;", value.as_ref());
        let attrs = syn::parse_str::<syn::DeriveInput>(&template).unwrap().attrs;
        Self { value: attrs }
    }
}

impl ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.value;
        tokens.extend(quote! { #(#attrs)* })
    }
}
