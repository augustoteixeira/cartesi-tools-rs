use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{quote, spanned::Spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, ItemFn, Result as SynResult,
};

#[derive(Debug)]
struct Args {
    ignore: bool,
    kind: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            ignore: false,
            kind: None,
        }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut pa = Args::default();

        while !input.is_empty() {
            let t = input.parse()?;
            match t {
                TokenTree::Ident(i) if i.to_string() == "ignore" => pa.ignore = true,

                TokenTree::Ident(i) if i.to_string() == "kind" => match input.parse()? {
                    TokenTree::Group(g) => {
                        let arr: Vec<_> = g.stream().into_iter().collect();
                        if arr.len() != 1 {
                            return Err(syn::Error::new(
                                g.span(),
                                "argument `kind` must have one argument",
                            ));
                        }

                        if let TokenTree::Literal(kind) = &arr[0] {
                            pa.kind = Some(kind.to_string());
                        }
                    }

                    x => {
                        return Err(syn::Error::new(
                            x.span(),
                            format!("unrecognized kind `{x}`"),
                        ))
                    }
                },

                TokenTree::Punct(p) if p.as_char() == ',' => (),

                x => {
                    return Err(syn::Error::new(
                        x.span(),
                        format!("unrecognized argument `{x}`"),
                    ));
                }
            }
        }

        Ok(pa)
    }
}

#[proc_macro_attribute]
pub fn test_dapp(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_args = parse_macro_input!(args as Args);
    let test_fn = parse_macro_input!(input as ItemFn);
    let name = &test_fn.sig.ident;
    let test_name = name.to_string();

    if test_fn.sig.inputs.len() != 0 {
        return syn::Error::new(
            test_fn.sig.__span(),
            format!("test function `{}` must have no arguments", name),
        )
        .into_compile_error()
        .into();
    }

    let ignore = parsed_args.ignore;
    let kind = if let Some(k) = parsed_args.kind {
        quote! { Some(#k) }
    } else {
        quote! { Some }
    };

    let expanded = quote! {
        #test_fn

        testsi::inventory::submit! {
            testsi::TestCase { name: #test_name, function: #name, ignore: #ignore, kind: #kind  }
        }
    };

    TokenStream::from(expanded)
}
