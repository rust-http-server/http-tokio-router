mod route;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    route::route(args, input)
}