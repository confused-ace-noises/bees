use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Token, parse::Parse, punctuated::Punctuated};

pub(crate) fn chain_impl(chain: Chain) -> syn::Result<TokenStream> {
    chain.tokenize()
}

pub(crate) fn pipe_impl(Pipe { members }: Pipe) -> syn::Result<TokenStream> {
    if members.len() < 2 {
        return Err(syn::Error::new(
            Span::call_site(),
            "pipe! only accepts 2 or more arguments",
        ));
    }

    let mut iter = members.into_iter().rev();

    // ? can be unwrapped because it's checked before
    let ChainMember {
        ty: last,
        try_token: last_try_token,
    } = iter.next().unwrap();
    let ChainMember {
        ty: second_to_last,
        try_token: first_try_token,
    } = iter.next().unwrap();

    if let Some(token) = last_try_token {
        return Err(syn::Error::new_spanned(
            token,
            "Cannot use the ~ or try operator on the last `Handler`, as there is nothing to chain it to",
        ));
    }

    let mut tokens = if first_try_token.is_some() {
        quote! { ::bees::handlers::TryChain(#second_to_last, #last) }
    } else {
        quote! { ::bees::handlers::Chain(#second_to_last, #last) }
    };

    for ChainMember { ty, try_token } in iter {
        if try_token.is_some() {
            tokens = quote! { ::bees::handlers::TryChain(#ty, #tokens) };
        } else {
            tokens = quote! { ::bees::handlers::Chain(#ty, #tokens) };
        }
    }

    Ok(tokens)
}

fn wrap_in_chain(ty: &syn::Type, to_wrap: &mut TokenStream) {
    *to_wrap = quote! { ::bees::handlers::Chain<#ty, #to_wrap> }
}

fn wrap_in_try_chain(ty: &syn::Type, to_wrap: &mut TokenStream) {
    *to_wrap = quote! { ::bees::handlers::TryChain<#ty, #to_wrap> }
}

#[derive(Debug)]
struct ChainMember<T: Parse> {
    ty: T,
    try_token: Option<FallibleToken>,
}

#[derive(Debug)]
enum FallibleToken {
    Tilde(Token![~]),
    Try(Token![try]),
}

impl quote::ToTokens for FallibleToken {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FallibleToken::Tilde(tilde) => tilde.to_tokens(tokens),
            FallibleToken::Try(r#try) => r#try.to_tokens(tokens),
        }
    }
}

impl<T: Parse> Parse for ChainMember<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let try_token = if input.peek(Token![~]) {
            input.parse::<Token![~]>()?;
            Some(FallibleToken::Tilde(<Token![~]>::default()))
        } else if input.peek(Token![try]) {
            input.parse::<Token![try]>()?;
            Some(FallibleToken::Try(<Token![try]>::default()))
        } else {
            None
        };

        let ty = input.parse::<T>()?;
        Ok(Self { ty, try_token })
    }
}

#[derive(Debug)]
pub(crate) struct Chain {
    members: Punctuated<ChainMember<syn::Type>, Token![,]>,
}

impl Parse for Chain {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            members: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

impl Chain {
    pub(crate) fn tokenize(&self) -> syn::Result<TokenStream> {
        let members = &self.members;

        if members.len() < 2 {
            return Err(syn::Error::new(
                Span::call_site(),
                "chain! only accepts 2 or more arguments",
            ));
        }

        let mut iter = members.into_iter().rev();

        // ? can be unwrapped because it's checked before
        let ChainMember {
            ty: last,
            try_token: last_try_token,
        } = iter.next().unwrap();
        let ChainMember {
            ty: second_to_last,
            try_token: first_try_token,
        } = iter.next().unwrap();

        if let Some(token) = last_try_token {
            return Err(syn::Error::new_spanned(
                token,
                "Cannot use the ~ or try operator on the last `Handler`, as there is nothing to chain it to",
            ));
        }

        let mut tokens = if first_try_token.is_some() {
            quote! { ::bees::handlers::TryChain<#second_to_last, #last> }
        } else {
            quote! { ::bees::handlers::Chain<#second_to_last, #last> }
        };

        for ChainMember { ty, try_token } in iter {
            if try_token.is_some() {
                wrap_in_try_chain(ty, &mut tokens);
            } else {
                wrap_in_chain(ty, &mut tokens);
            }
        }

        Ok(tokens)
    }

    pub(crate) fn tokenize_pipely(&self) -> syn::Result<TokenStream> {
        let members = &self.members;

        if members.len() < 2 {
            return Err(syn::Error::new(
                Span::call_site(),
                "chain! only accepts 2 or more arguments",
            ));
        }

        let mut iter = members.into_iter().rev();

        // ? can be unwrapped because it's checked before
        let ChainMember {
            ty: last,
            try_token: last_try_token,
        } = iter.next().unwrap();
        let ChainMember {
            ty: second_to_last,
            try_token: first_try_token,
        } = iter.next().unwrap();

        if let Some(token) = last_try_token {
            return Err(syn::Error::new_spanned(
                token,
                "Cannot use the ~ or try operator on the last `Handler`, as there is nothing to chain it to",
            ));
        }

        let mut tokens = if first_try_token.is_some() {
            quote! { ::bees::handlers::TryChain(#second_to_last, #last) }
        } else {
            quote! { ::bees::handlers::Chain(#second_to_last, #last) }
        };

        for ChainMember { ty, try_token } in iter {
            if try_token.is_some() {
                tokens = quote! { ::bees::handlers::TryChain(#ty, #tokens) }
            } else {
                tokens = quote! { ::bees::handlers::Chain(#ty, #tokens) }
            }
        }

        Ok(tokens)
    }

    pub(crate) unsafe fn new_raw(members: Punctuated<ChainMember<syn::Type>, Token![,]>) -> Self {
        Self { members }
    }
}

pub(crate) struct Pipe {
    members: Punctuated<ChainMember<syn::Expr>, Token![,]>,
}

impl Parse for Pipe {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            members: Punctuated::parse_terminated(input)?,
        })
    }
}
