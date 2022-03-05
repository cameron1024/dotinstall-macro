use proc_macro2::TokenStream;
use syn::parse2;

use self::codegen::generate_installer;

mod codegen;
mod parse;

pub fn installer(tokens: TokenStream) -> TokenStream {
    match parse2(tokens) {
        Ok(installer) => generate_installer(&installer),
        Err(e) => e.to_compile_error(),
    }
}
