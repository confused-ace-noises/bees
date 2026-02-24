use deluxe::{HasAttributes, ParseAttributes};
// use proc_macro_crate::{FoundCrate, crate_name};
use quote::{quote, quote_spanned};
use syn::{Token, punctuated::Punctuated, spanned::Spanned};

pub(crate) fn procs_derive_impl(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ProcessAttrs(procs) = ProcessAttrs::parse_attributes(&input)?;
    
    let ident = input.ident;
    
    let impls = procs.into_iter().map(|proc_path| {
        let span = proc_path.span();
        quote_spanned! {span=> 
            #[automatically_derived]
            impl ::bees::endpoint::EndpointProcessor<<#proc_path as ::bees::endpoint::Process>::ProcessOutput> for #ident {
                type Process = #proc_path;

                fn refine(proc_output: <Self::Process as Process>::ProcessOutput, _: &Self::CallContext) -> impl ::std::future::Future<Output = <Self::Process as Process>::ProcessOutput> {
                    ::std::future::ready(proc_output)
                }
            }
        }
    });


    let all_impls = quote! {
        #(#impls)*
    };

    Ok(all_impls)
}

#[derive(Debug)]
// #[deluxe(attributes(process))]
struct ProcessAttrs(syn::punctuated::Punctuated<syn::Path, Token![,]>);

impl ProcessAttrs {
    fn path_match(path: &syn::Path) -> bool {
        path.is_ident("process")
    }
}

impl<'t, T: HasAttributes> ParseAttributes<'t, T> for ProcessAttrs {
    fn path_matches(path: &syn::Path) -> bool {
        Self::path_match(path)
    }

    fn parse_attributes(obj: &'t T) -> deluxe::Result<Self> {
        let mut result = Punctuated::<syn::Path, Token![,]>::new();
        
        for attr in obj.attrs() {
            if !Self::path_match(attr.path()) {
                continue;
            }

            // Ensure it's #[process(...)]
            let meta = attr.meta.require_list()?;

            // Parse the contents inside (...)
            let parsed: Punctuated<syn::Path, Token![,]> =
                meta.parse_args_with(Punctuated::parse_terminated)?;

            result.extend(parsed);
        }

        Ok(ProcessAttrs(result))
    }
}