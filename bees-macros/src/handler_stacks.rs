use deluxe::{HasAttributes, ParseAttributes, ParseMetaItem};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Block, Token, parse::{Parse, ParseStream, discouraged::Speculative}, punctuated::Punctuated, token};

use crate::Chain;

pub(crate) fn handler_stacks_impl(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let FullSpec(stacks) = deluxe::parse_attributes(&input)?;

    let ident = input.ident;

    let mut final_tokens = TokenStream::new();

    for HandlerSpec { block, handler_list: chain, output_type } in stacks {
        let tokens_chain = chain.tokenize()?;

        // let quote_impl = quote! { impl ::bees::endpoint::HandlerStack<<#tokens_chain as ::bees::handlers::Handler>::Output> for #ident };
        let quote_impl = quote! { impl ::bees::endpoint::HandlerStack<#output_type> for #ident };
        let handlers_type = quote! { type Handlers = #tokens_chain; };

        let handlers = quote! {

            async fn handlers(ctx: &mut <Self as ::bees::endpoint::EndpointInfo>::CallContext) -> Result<Self::Handlers, Box<dyn ::std::error::Error + Send + Sync>> {Ok(#block)}
        };  


        let composed = quote! {
            #[automatically_derived]
            #quote_impl {
                #handlers_type

                #handlers
            }
        };

        final_tokens = quote! {
            #final_tokens

            #composed
        };
    }

    Ok(final_tokens)
}

struct FullSpec(Punctuated<HandlerSpec, Token![;]>);

impl<'t, T: HasAttributes + std::fmt::Debug> ParseAttributes<'t, T> for FullSpec {
    fn path_matches(path: &syn::Path) -> bool {
        path.is_ident("stacks")
    }

    fn parse_attributes(obj: &'t T) -> deluxe::Result<Self> {
        let mut punct = Punctuated::<HandlerSpec, Token![;]>::new();

        for attr in obj.attrs() {
            if !<Self as ParseAttributes<'t, T>>::path_matches(attr.path()) {
                continue;
            }

            let meta = attr.meta.require_list()?;

            let parsed: Punctuated<HandlerSpec, Token![;]> =
                meta.parse_args_with(Punctuated::parse_separated_nonempty)?;

            punct.extend(parsed);
        }

        Ok(Self(punct))
    }
}

#[derive(Debug)]
struct HandlerSpec {
    output_type: syn::Type,
    // _colon: Token![:],
    block: Block,
    // _arrow: Token![->],
    handler_list: HandlerList,
}

impl Parse for HandlerSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let output_type = input.parse::<syn::Type>()?;
        let _x = input.parse::<Token![:]>()?;

        let (chain, block): (HandlerList, Block);
    
        // try to collapse the block and the output
        if !input.peek(token::Brace) {
            chain = input.parse::<HandlerList>()?;

            let tokens = chain.tokenize_pipely()?;
            let quote = quote! { {#tokens} };

            block = syn::parse2::<Block>(quote)?;
        } else {
            block = input.parse::<Block>()?;
            let _  = input.parse::<syn::Token![->]>()?;
            chain = input.parse::<HandlerList>()?;
    
        }

        let s = Self { output_type, block, handler_list: chain };

        Ok(s)
    }
}

impl ParseMetaItem for HandlerSpec {
    fn parse_meta_item(input: ParseStream, _mode: deluxe::ParseMode) -> syn::Result<Self> {
        Self::parse(input)
    }
}

#[derive(Debug)]
pub enum HandlerList {
    Single(syn::Type),
    Chain(Chain),
}

impl Parse for HandlerList {
    // TODO: performance optimization, don't parse the whole thing with Punctuated,
    // TODO: only the first element is actually needed  
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let forked = input.fork();

        let punct = Punctuated::<syn::Type, Token![,]>::parse_separated_nonempty(&forked);

        if let Ok(ok_punct) = punct {

            if ok_punct.is_empty() {
                return Err(syn::Error::new(Span::call_site(), "#stacks only accepts 1 or more arguments"));
            }
            
            if ok_punct.len() == 1 {
                // ? checked  ^
                input.advance_to(&forked);
                let ty = ok_punct.into_iter().next().unwrap();
                Ok(HandlerList::Single(ty))
            } else {
                let chain= input.parse::<Chain>()?;
        
                Ok(HandlerList::Chain(chain))
            } 
        } else {
            let chain= input.parse::<Chain>()?;
    
            Ok(HandlerList::Chain(chain))
        }


    }
}

impl HandlerList {
    pub fn tokenize(&self) -> syn::Result<TokenStream> {
        match self {
            HandlerList::Single(ty) => Ok(ty.to_token_stream()),
            HandlerList::Chain(chain) => chain.tokenize(),
        }
    }

    pub fn tokenize_pipely(&self) -> syn::Result<TokenStream> {
        match self {
            HandlerList::Single(ty) => Ok(ty.to_token_stream()),
            HandlerList::Chain(chain) => chain.tokenize_pipely(),
        }
    }
}

 
/*


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
    */