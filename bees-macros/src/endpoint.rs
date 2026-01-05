use deluxe::ParseAttributes;
use quote::quote;
use syn::{Expr, LitStr};

pub fn endpoint_derive_impl(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = input.ident;
    
    let EndpointAttrs {
        name,
        path: url,
        record_name,
        http_verb,
        capabilities,
        processors,
    } = EndpointAttrs::parse_attributes(&input.attrs)?;

    let name = name.unwrap_or(LitStr::new(&ident.to_string(), ident.span()));
    let capabilities = capabilities.unwrap_or_default();
    
    if processors.is_empty() {
        return Err(
            syn::Error::new(
                proc_macro2::Span::call_site(), 
                "processor list cannot be empty"
            )
        )
    }

    Ok(quote! {
        impl #ident {
            pub fn name() -> ::std::string::String {
                ::std::string::String::from(#name)
            }

            pub fn path() -> ::std::string::String {
                ::std::string::String::from(#url)
            }

            pub fn record_name() -> ::std::string::String {
                ::std::string::String::from(#record_name)
            }

            pub fn http_verb() -> ::bees::net::client::HttpVerb {
                #http_verb
            }

            pub fn capabilities() 
                -> ::std::sync::Arc<
                    [::std::boxed::Box<
                        dyn ::bees::capability::Capability
                    >]
                >
            {
                ::std::sync::Arc::new([
                    #( 
                        ::bees::capability::IntoBoxedCapability::into_boxed_capability(#capabilities)
                    ),*
                ])
            }

            pub fn endpoint_builder() -> ::bees::endpoint::EndpointBuilder {
                let template = ::bees::endpoint::EndpointTemplate {
                    record_name: Self::record_name(),
                    name: Self::name(),
                    path: ::bees::utils::FormatString::new(&Self::path()),
                    http_verb: Self::http_verb(),
                    capabilities: Self::capabilities(),
                };

                let mut builder = ::bees::endpoint::Endpoint::builder_template(template);

                #(
                    builder.push_endpoint_output(#processors);
                )*

                builder
            }

            pub fn endpoint() -> ::bees::endpoint::Endpoint {
                Self::endpoint_builder().build()
            }
        }

        impl ::core::convert::From<#ident> for ::bees::context::pre_context::ContextMod {
            fn from(_: #ident) -> Self {
                fn __auto_add_endpoint(
                    ctx: &mut ::bees::context::context::Context
                ) {
                    let builder = #ident::endpoint_builder();

                    // ensure record exists with unwrap 
                    let record = ctx
                        .records
                        .get_record(#record_name)
                        .unwrap_or_else(|| panic!("derive(Endpoint): couldn't find Record `{}`, needed by Endpoint `{}`", {#record_name}, {#name}));

                    let info = <::bees::record::RecordInfo as ::core::convert::From<::bees::record::Record>>::from(record.clone());
                    
                    // existence of the record was checked earlier
                    let endpoint = unsafe {
                        builder.build_unchecked(info.constant_url().clone(), info.capabilities().clone())
                    };

                    record.add_endpoint(endpoint);
                }
                
                Self::new(
                    ::bees::context::pre_context::ContextModPriority::Endpoint,
                    ::std::boxed::Box::new(__auto_add_endpoint),
                )
            }
        }
    })
}

#[derive(Debug, ParseAttributes)]
#[deluxe(attributes(endpoint))]
struct EndpointAttrs {
    name: Option<LitStr>,
    path: LitStr,
    record_name: LitStr,
    http_verb: Expr,
    capabilities: Option<Vec<Expr>>,
    processors: Vec<Expr>,
}