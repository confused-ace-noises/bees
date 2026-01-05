use proc_macro::TokenStream;
use syn::{Data, DeriveInput};
use crate::{endpoint::endpoint_derive_impl, record::record_derive_impl};
mod record;
mod endpoint;

#[proc_macro_derive(Record, attributes(record))]
pub fn record(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match validate_unit_struct_no_generics(&input) {
        Ok(_) => {},
        Err(e) => return e.into_compile_error().into(),
    }

    match record_derive_impl(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_derive(Endpoint, attributes(endpoint))]
pub fn endpoint(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    
    match validate_unit_struct_no_generics(&input) {
        Ok(_) => {},
        Err(e) => return e.into_compile_error().into(),
    }

    match endpoint_derive_impl(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn validate_unit_struct_no_generics(input: &DeriveInput) -> syn::Result<()> {
    // no generics
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Record must not have generics",
        ));
    }

    // must be a unit struct
    match &input.data {
        Data::Struct(s) if matches!(s.fields, syn::Fields::Unit) => {}
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Record must be a unit struct",
            ));
        }
    }

    Ok(())
}
