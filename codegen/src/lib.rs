mod autodoc;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn autodoc(attr: TokenStream, item: TokenStream) -> TokenStream {
    unwrap!(autodoc::handle_autodoc(attr, item))
}
