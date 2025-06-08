use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, Pat, PatIdent};

pub fn route(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let original_name = input_fn.sig.ident.clone();

    let mut new_name = original_name.to_string();
    new_name.push_str("_inner");
    let new_name = Ident::new(&new_name, input_fn.sig.ident.span());
    input_fn.sig.ident = new_name.clone();

    // let mut ctx_type: Option<Box<Type>> = None;
    // let mut req_type: Option<&PatType> = None;

    let mut let_bindings = Vec::new();
    let mut param_exprs = Vec::new();

    for (i, param) in input_fn.sig.inputs.iter().enumerate() {
        match param {
            FnArg::Typed(pat_type) => {
                let var_ident = if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                    ident.clone()
                } else {
                    format_ident!("param{}", i)
                };

                let ty = &*pat_type.ty;

                let let_stmt = quote! {
                    let #var_ident: #ty = match <#ty as http_tokio_router::extractors::FromRequest>::from_req(&*r, b).await {
                        Ok(val) => val,
                        Err(e) => return Err(e),
                    };
                };
                let_bindings.push(let_stmt);
                param_exprs.push(var_ident);
            }
            _ => {}
        }
    }

    // let tag_vec = tags.iter().map(|tag| quote! { #tag.to_string() });

    let output = quote! {
        pub fn #original_name<'a>(r: &'a http_tokio::Request, b: &'a http_tokio::BodyReader) -> http_tokio_router::result::HandlerResult<'a> {
            #input_fn
            Box::pin(async move {
                #(#let_bindings)*
                let __returned = #new_name(#(#param_exprs),*).await;
                http_tokio_router::result::IntoRouteResult::into(__returned)
            })
        }
    };

    output.into()
}

// struct RouteArgs {
//     pattern: String,
//     // tags: Vec<String>,
// }

// impl Parse for RouteArgs {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let pattern: LitStr = input
//             .parse()
//             .map_err(syn_err_mapper("Failed to parse pattern string"))?;

//         // let mut tags = vec![];

//         // if !input.is_empty() {
//         //     input
//         //         .parse::<Token![,]>()
//         //         .map_err(syn_err_mapper("Arguments should be comma separated"))?;
//         //     let ident = input.parse::<Ident>()?;
//         //     if ident != "tags" {
//         //         return Err(syn::Error::new_spanned(
//         //             ident,
//         //             "Unexpected argument, expected `tags(...)`",
//         //         ));
//         //     }

//         //     let content;
//         //     syn::parenthesized!(content in input);
//         //     let tag_strs = content.parse_terminated(<LitStr as Parse>::parse, Token![,])?;
//         //     tags = tag_strs.into_iter().map(|s| s.value()).collect();
//         // }

//         Ok(RouteArgs {
//             pattern: pattern.value(),
//             // tags,
//         })
//     }
// }

// fn syn_err_mapper(message: &'static str) -> impl Fn(syn::Error) -> syn::Error {
//     return move |e: syn::Error| syn::Error::new(e.span(), format!("{message}: {e}"));
// }
