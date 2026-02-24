use deluxe::ParseAttributes;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub(crate) fn record_impl(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    
    let RecordArgs { path, capabilities } = RecordArgs::parse_attributes(&input)?;
    
    let capabilities = capabilities.unwrap_or(Vec::new());
    
    let ident = input.ident;

    let path_span = path.span();
    let ident_span = ident.span();
    
    let shared_url = quote_spanned! {path_span=> const SHARED_URL: &str = #path; };
    let impl_piece = quote_spanned! {ident_span=> 
        #[automatically_derived]
        impl ::bees::record::Record for #ident 
    };
    
    let shared_caps = make_capabilities(capabilities);

    let implementation = quote! {#impl_piece {
        #shared_url
        fn shared_caps() -> Arc<[Box<dyn Capability>]> {
            ::std::sync::Arc::new([ #(#shared_caps),* ])
        } 
    }};
    
    Ok(implementation)
}

pub(crate) fn make_capabilities(capabilities: Vec<syn::Expr>) -> impl Iterator<Item = proc_macro2::TokenStream> {
    capabilities.into_iter().map(|expr| {
        let expr_span = expr.span();

        quote_spanned! {expr_span=> ::std::boxed::Box::new(#expr) as ::std::boxed::Box<dyn ::bees::capability::Capability>}
    })
}

#[derive(Debug, ParseAttributes)]
#[deluxe(attributes(record))]
struct RecordArgs {
    path: syn::LitStr,
    capabilities: Option<Vec<syn::Expr>>
}
