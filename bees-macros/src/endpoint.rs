use deluxe::ParseAttributes;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned};

use crate::record::make_capabilities;

pub(crate) fn endpoint_derive(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let EndpointAttributes {
        record,
        http_method: http_verb,
        capabilities,
        // processors,
        path,
        // handler: HandlerSpec {block, output, ..},
        modify_url,
    } = EndpointAttributes::parse_attributes(&input)?;

    let ident = input.ident;
    let ident_span = ident.span();
    let impl_piece = quote_spanned! {ident_span=> 
        #[automatically_derived]
        impl ::bees::endpoint::EndpointInfo for #ident 
    };

    let record_span = record.span();
    let record_piece = quote_spanned! {record_span=> type Record = #record; };

    let path_span = path.span();
    let path_piece = quote_spanned! {path_span=> const PATH: &str = #path; };

    let http_verb_span = http_verb.span();
    let http_verb_piece = quote_spanned! {http_verb_span=> 
        #[allow(clippy::manual_async_fn)]
        fn http_method(_: &mut Self::CallContext) -> impl Future<Output = HttpMethod> + Send { async move { #http_verb } } 
    };

    let capability_pieces = make_capabilities(capabilities);

    let path = quote! { ::bees::capability::Capability };

    let capability_fn = quote! {
        fn capabilities(_: &mut Self::CallContext) -> ::std::sync::Arc<[Box<dyn #path>]> {
            ::std::sync::Arc::new([ #(#capability_pieces),* ])
        }
    };

    // let handler_type_span = output.span();
    // let handler_type_piece = quote_spanned! {handler_type_span=> type EndpointHandler = #output; };

    // let handler_expr_span = output.span();
    // let handler_expr_piece = quote_spanned! {handler_expr_span=> fn endpoint_handler(_: &Self::CallContext) -> Self::EndpointHandler #block };

    let url_mod_fn_body = match modify_url {
        Some(url_mod_fn) => {
            let url_mod_fn_span = url_mod_fn.span();
            quote_spanned! {url_mod_fn_span=> #url_mod_fn(____url___)}
        },
        None => {
            quote! {::std::future::ready(____url___)}
        },
    };

    let url_mod_fn = quote! {
        #[allow(clippy::manual_async_fn)]
        fn modify_url(____url___: ::bees::re_exports::url::Url, _: &mut Self::CallContext) -> impl ::std::future::Future<Output = ::bees::re_exports::url::Url> + ::std::marker::Send {
            #url_mod_fn_body
        }
    };

    // let proc_impls = processors.into_iter().map(|proc_path| {
    //     let span = proc_path.span();
    //     quote_spanned! {span=> 
    //         #[automatically_derived]
    //         impl ::bees::endpoint::EndpointProcessor<<#proc_path as ::bees::endpoint::Process>::ProcessOutput> for #ident {
    //             type Process = #proc_path;

    //             #[allow(clippy::manual_async_fn)]
    //             fn refine(proc_output: <Self::Process as Process>::ProcessOutput, _: &Self::CallContext) -> impl ::std::future::Future<Output = <Self::Process as Process>::ProcessOutput> {
    //                 ::std::future::ready(proc_output)
    //             }
    //         }
    //     }
    // });

    let result = quote! {
        #impl_piece {
            #path_piece

            // #handler_type_piece
            #record_piece
            type CallContext = ();

            #http_verb_piece
            #capability_fn

            // #handler_expr_piece
            #url_mod_fn
        }

        // #(#proc_impls)*
    };

    Ok(result)
}

#[derive(Debug, ParseAttributes)]
#[deluxe(attributes(endpoint))]
struct EndpointAttributes {
    record: syn::Type,
    http_method: syn::Expr,
    #[deluxe(default = Vec::new())]
    capabilities: Vec<syn::Expr>,
    path: syn::LitStr,
    // handler: HandlerSpec,
    modify_url: Option<syn::Type>,
}