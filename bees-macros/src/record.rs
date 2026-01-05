use deluxe::ParseAttributes;
use quote::quote;
use syn::LitStr;

pub fn record_derive_impl(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;

    match &input.data {
        syn::Data::Struct(_) => {}
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Record can only be derived for structs",
            ))
        }
    };

    let RecordAttrs { 
        name, 
        url: base_url,
        capabilities
    } = RecordAttrs::parse_attributes(&input.attrs)?;

    let record_name = name.unwrap_or(syn::LitStr::new(&ident.to_string(), ident.span()));
    
    let capabilities = capabilities.unwrap_or_default();

    Ok(quote! {
        impl #ident {
            pub fn record_name() -> ::std::string::String {
                ::std::string::String::from(#record_name)
            }

            pub fn base_url() -> ::std::string::String {
                ::std::string::String::from(#base_url)
            }

            pub fn shared_capabilities() 
                -> ::std::sync::Arc<
                    [::std::boxed::Box<
                        dyn ::bees::capability::Capability
                    >]
                >
            {
                ::std::sync::Arc::new([
                    #( ::bees::capability::IntoBoxedCapability
                        ::into_boxed_capability(#capabilities)
                    ),*
                ])
            }

            pub fn record() -> ::bees::record::Record {
                ::bees::record::Record::new(
                    Self::record_name(),
                    Self::base_url(),
                    Self::shared_capabilities(),
                )
            }
        }

        impl ::core::convert::From<#ident> for ::bees::context::pre_context::ContextMod {
            fn from(_: #ident) -> Self {
                fn __auto_add_record(
                    ctx: &mut ::bees::context::context::Context
                ) {
                    ctx.records.add_record(#ident::record())
                }
                
                Self::new(
                    ::bees::context::pre_context::ContextModPriority::Record,
                    ::std::boxed::Box::new(__auto_add_record),
                )
            }
        }
    })
}

#[derive(Debug, ParseAttributes)]
#[deluxe(attributes(record))]
struct RecordAttrs {
    name: Option<LitStr>,
    url: LitStr,
    capabilities: Option<Vec<syn::Expr>>,
}
