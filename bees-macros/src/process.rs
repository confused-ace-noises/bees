use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{FnArg, Ident, PatType, Signature, Type, TypePath, spanned::Spanned};

pub(crate) fn attr_process(mut func: syn::ItemFn) -> syn::Result<TokenStream> {
    
    let cloned_sig = func.sig.clone();
    
    let sig = &mut func.sig;
    let vis = &func.vis;

    let (input, output) = check_signature(&cloned_sig)?;
    let output_span = sig.output.span();

    let mut hijacked_name = String::from("_");
    hijacked_name.push_str(&sig.ident.to_string());

    let ident = Ident::new(&hijacked_name, sig.ident.span());

    hijacked_name.push_str("__");

    let input_ident = Ident::new(&hijacked_name, Span::call_site());

    sig.ident = ident.clone();

    let name = &cloned_sig.ident;
    // let name_span = name.span();

    let is_async = cloned_sig.asyncness.is_some();

    // let struct_ = quote! {name.span()=> #vis struct #name;};

    // let struct_impl = quote! {name_span=> impl ::bees::endpoint::Process for #name};

    let struct_ = quote! {#vis struct #name;};

    let struct_impl = quote! {impl ::bees::endpoint::Process for #name};

    let output_type = quote_spanned! {output_span=> type ProcessOutput = #output; };

    let call = {
        if is_async {
            quote!{#ident(#input_ident)}
        } else {
            quote! {::std::future::ready(#ident(#input_ident))}
        }
    };

    let fn_process = quote! {
            fn process(#input_ident: #input) -> impl Future<Output = Self::ProcessOutput> + Send {
            #[allow(non_snake_case)]
            #func

            #call
        }
    };

    let finished = quote! {
        #struct_
        #struct_impl {
            #output_type

            #fn_process
        }
    };

    Ok(finished)
}

fn check_signature(sig: &Signature) -> syn::Result<(&syn::Path, &Type)> {
    if sig.inputs.len() != 1 {
        return Err(syn::Error::new_spanned(&sig.inputs, "Expected exactly one argument of type `reqwest::Response` for a `Processor`."))
    }

    let input = &sig.inputs[0];
    // let input_name = input.
    let input_path: &syn::Path;

    match input {
        FnArg::Typed(PatType { ty, .. }) => {
            match &**ty {
                syn::Type::Path(TypePath { path, ..}) => {
                    let last_segment = path.segments.last().ok_or(syn::Error::new_spanned(path, "path should not be empty."))?;
                    
                    if last_segment.ident != "Response" {
                        return Err(syn::Error::new_spanned(last_segment, "A `Processor` must only take one argument and it must be of type `Response`."));
                    }

                    input_path = path;
                },
                _ => return Err(syn::Error::new_spanned(input, "A `Processor` must only take one argument and it must be of type `Response`.")),
            }
            
        },
        _ => return Err(syn::Error::new_spanned(input, "A `Processor` must only take one argument and it must be of type `Response`.")),
    }

    match &sig.output {
        syn::ReturnType::Default => Err(syn::Error::new_spanned(&sig.output, "The return type of a `Processor` must be explicit. If you really intended for this `Processor` to return (), add `-> ()` as the return type")),
        syn::ReturnType::Type(.., ty) => {
            let ty = &**ty;
            
            if let Type::ImplTrait(_) = ty {
                return Err(syn::Error::new_spanned(ty, "`impl Trait` in this position is not allowed in stable rust."))
            }

            Ok((input_path, ty))
        }
    }
}