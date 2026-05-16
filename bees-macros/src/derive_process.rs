use deluxe::{HasAttributes, ParseAttributes, ParseMetaItem};
// use proc_macro_crate::{FoundCrate, crate_name};
use quote::{quote, quote_spanned};
use syn::{Block, Token, Type, parse::ParseStream, punctuated::Punctuated, spanned::Spanned};

pub(crate) fn handler_stacks_impl(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ProcessAttrs(procs) = ProcessAttrs::parse_attributes(&input)?;
    
    let ident = input.ident;
    
    let impls = procs.into_iter().map(|proc_path| {
        let span = proc_path.span();
        quote_spanned! {span=> 
            #[automatically_derived]
            impl ::bees::endpoint::HandlerStack<<#proc_path as ::bees::endpoint::Process>::ProcessOutput> for #ident {
                type Process = #proc_path;
            }
        }
    });


    let all_impls = quote! {
        #(#impls)*
    };

    Ok(all_impls)
}

#[derive(Debug)]
struct HandlerSpec {
    block: Block,
    _arrow: Token![->],
    output: Type,
}

impl ParseMetaItem for HandlerSpec {
    fn parse_meta_item(input: ParseStream, _mode: deluxe::ParseMode) -> syn::Result<Self> {
        let block = input.parse::<Block>()?;
        let _arrow = input.parse::<syn::Token![->]>()?;
        let output: syn::Type = input.parse()?;

        Ok(Self { block, _arrow, output })
    }
}

#[derive(Debug)]
// #[deluxe(attributes(process))]
struct ProcessAttrs(syn::punctuated::Punctuated<HandlerSpec, Token![,]>);

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
            let parsed: Punctuated<HandlerSpec, Token![,]> =
                meta.parse_args_with(Punctuated::parse_terminated)?;

            result.extend(parsed);
        }

        Ok(ProcessAttrs(result))
    }
}