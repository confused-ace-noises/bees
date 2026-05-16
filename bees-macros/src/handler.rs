use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    FnArg, Ident, Pat, PatIdent, PatType, Signature, Type, spanned::Spanned,
};

pub(crate) fn attr_handler(mut func: syn::ItemFn) -> syn::Result<TokenStream> {
    let cloned_sig = func.sig.clone();

    let sig = &mut func.sig;
    let vis = &func.vis;

    let Abstraction {
        input,
        output: output_type,
        others,
        call_args,
    } = check_signature(&cloned_sig)?;

    sig.inputs.iter_mut().for_each(|arg| {
        match arg {
            FnArg::Typed(typed) => {
                typed.attrs.retain(|attr| {
                    !attr
                        .meta
                        .require_path_only()
                        .map(|p| p.is_ident("input"))
                        .unwrap_or(false)
                });
            },

            _ => unreachable!("checked before")
        }
    });

    let input_span = input.span();
    let input_type = &*input.ty;

    let is_async = cloned_sig.asyncness.is_some();
    let name = &cloned_sig.ident;

    let (impl_generics, type_gens, where_clause) = sig.generics.split_for_impl();

    // ******** STRUCT ********
    let struct_upper = quote! {#vis struct #name #type_gens #where_clause};
    let struct_args = {
        if others.is_empty() {
            quote! {;}
        } else {
            let quotes = others
                .iter()
                .map(|PatType { pat, ty, .. }| quote! {#pat: #ty})
                .collect::<Vec<_>>();
            quote! { { #(#quotes),* } }
        }
    };

    let struct_whole = quote! { #struct_upper #struct_args };
    // ******** STRUCT ********

    let output_span = sig.output.span();

    // ******** HIJACK IDENT ********
    let mut hijacked_name = String::from("_");
    hijacked_name.push_str(&sig.ident.to_string());

    let hijack_ident = Ident::new(&hijacked_name, sig.ident.span());

    sig.ident = hijack_ident.clone();
    // ******** HIJACK IDENT ********

    let call = {
        if is_async {
            quote! {#hijack_ident( #(#call_args),* )}
        } else {
            quote! {::std::future::ready(#hijack_ident( #(#call_args),* ))}
        }
    };

    let struct_impl =
        quote! { impl #impl_generics ::bees::endpoint::Handler for #name #type_gens #where_clause };

    let input_associated = quote_spanned! {input_span=> type Input = #input_type; };
    let output_associated = quote_spanned! {output_span=> type Output = #output_type; };

    let fn_process = quote! {
        fn execute(&self, input: Self::Input) -> impl Future<Output = Self::Output> + Send {
            #[allow(non_snake_case)]
            #func

            #call
        }
    };

    let finished = quote! {
        #struct_whole
        #struct_impl {
            #input_associated
            #output_associated

            #fn_process
        }
    };

    Ok(finished)
}

fn check_signature(sig: &Signature) -> syn::Result<Abstraction<'_>> {
    let has_receiver = sig
        .inputs
        .iter()
        .any(|arg| matches!(arg, FnArg::Receiver(_)));

    if has_receiver {
        return Err(syn::Error::new_spanned(
            &sig.inputs,
            "This function must not contain any `self` arguments. If you need custom arguments, simply add them in plain and they will be added in the `Handler` struct.",
        ));
    }

    let mut iter = sig.inputs.iter();
    let mut input: Option<&PatType> = None;
    let mut others = Vec::new();
    let mut call_args = Vec::new();

    while let Some(FnArg::Typed(typed)) = iter.next() {
        if !matches!(*typed.pat, Pat::Ident(_)) {
            return Err(syn::Error::new_spanned(
                typed,
                "Only ident arguments are supported in this position",
            ));
        }

        let has_input_attr = typed.attrs.iter().any(|attr: &syn::Attribute| {
            let path = attr.meta.require_path_only();
            if let Ok(path) = path {
                path.is_ident("input")
            } else {
                false
            }
        });

        if has_input_attr {
            if input.is_none() {
                input = Some(typed);
                call_args.push(quote! { input }); // ? name of arg passed by execute()
            } else {
                return Err(syn::Error::new_spanned(
                    &sig.inputs,
                    "There must be only one #[input] attribute, which signals the type of the `Handler::Input` associated type.",
                ));
            }
        } else {
            let Pat::Ident(PatIdent { ident, .. }) = &*typed.pat else {
                unreachable!("this was checked at the start of the loop")
            };

            others.push(typed);

            call_args.push(quote! { self.#ident });
        }
    }


    if input.is_none() {
        return Err(syn::Error::new_spanned(
            &sig.inputs,
            "A `Handler` must contain exactly one `input` argument, signaled by the #[input] attribute.",
        ));
    }

    match &sig.output {
        syn::ReturnType::Default => Err(syn::Error::new_spanned(
            &sig.output,
            "The return type of a `Handler` must be explicit. If you really intended for this `Handler` to return (), add `-> ()` as the return statement",
        )),
        syn::ReturnType::Type(.., ty) => {
            let output_ty = &**ty;

            if let Type::ImplTrait(_) = output_ty {
                return Err(syn::Error::new_spanned(
                    output_ty,
                    "`impl Trait` in this position is not allowed in stable rust.",
                ));
            }

            let abs = Abstraction {
                input: input.unwrap(), // ? this will never panic, because input.is_none() is checked earlier
                output: output_ty,
                others,
                call_args,
            };

            Ok(abs)
        }
    }
}

struct Abstraction<'a> {
    input: &'a PatType,
    output: &'a Type,
    others: Vec<&'a PatType>,
    call_args: Vec<TokenStream>,
}
