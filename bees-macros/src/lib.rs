use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{ToTokens, quote};
use syn::{LitStr, parse_quote, spanned::Spanned};
/*
#[proc_macro]
pub fn format_string(stream: TokenStream) -> TokenStream {
    let parsed: LitStr = syn::parse_macro_input!(stream);

    let val = parsed.value();

    let mut chars = val.chars().peekable();

    let mut raw_sting_buffer = String::new();
    let mut parts: Vec<FormatStringPartMacro> = Vec::new();

    let found = crate_name("bees").expect("bees crate not found");

    let crate_path = match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, parsed.span());
            quote!(::#ident)
        }
    };

    'outer: while let Some(c) = chars.next() {
        match c {
            '<' => {
                if let Some(&'<') = chars.peek() {
                    let _ = chars.next();
                    raw_sting_buffer.push('<');
                    continue 'outer;
                } else {
                    parts.push(FormatStringPartMacro(false, raw_sting_buffer));
                    raw_sting_buffer = String::new();

                    let mut part = String::new();

                    'inner: while let Some(c_part) = chars.next() {
                        match c_part {
                            '>' => {
                                if let Some(&'>') = chars.peek() {
                                    let _ = chars.next();
                                    part.push('>');
                                    continue 'inner;
                                } else {
                                    break 'inner;
                                }
                            }

                            '<' => {
                                if let Some(&'<') = chars.peek() {
                                    let _ = chars.next();
                                    part.push('<');
                                    continue 'inner;
                                } else {
                                    return syn::Error::new(
                                        parsed.span(),
                                        "invalid formattable string: lone '<' inside formattable section (did you mean '<<'?)"
                                    ).to_compile_error().into();
                                }
                            }

                            a => part.push(a),
                        }
                    }

                    parts.push(FormatStringPartMacro(true, part));
                }
            }

            '>' => {
                if let Some(&'>') = chars.peek() {
                    let _ = chars.next();
                    raw_sting_buffer.push('>')
                } else {
                    return syn::Error::new(
                        parsed.span(),
                        "invalid formattable string in FormatString: unpaired \'>\' inside raw section (did you mean \'>>\'?)"
                    ).to_compile_error().into();
                }
            }

            c => raw_sting_buffer.push(c),
        }
    }

    if !raw_sting_buffer.is_empty() {
        parts.push(FormatStringPartMacro(false, raw_sting_buffer));
    }

    let quote = quote! {
        #crate_path::utils::format_string::FormatString::from_parts(Box::new([#(#parts),*]))
    };

    quote.into()
}

struct FormatStringPartMacro(bool, String);

impl ToTokens for FormatStringPartMacro {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // false means raw string
        let string = &self.1;

        let found = crate_name("bees").expect("bees crate not found");

        let crate_path = match found {
            FoundCrate::Itself => quote!(crate),
            FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, tokens.span());
                quote!(::#ident)
            }
        };

        if !self.0 {
            quote! {#crate_path::utils::format_string::FormattableStringPart::Raw(#string.to_string())}
                .to_tokens(tokens);
        } else {
            // true means resource
            quote! {#crate_path::utils::format_string::FormattableStringPart::ResourceReplace(#string.to_string())}.to_tokens(tokens);
        }
    }
}
*/