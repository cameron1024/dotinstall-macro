use proc_macro::TokenStream;

mod installer;

#[proc_macro]
pub fn installer(tokens: TokenStream) -> TokenStream {
    installer::installer(tokens.into()).into()
}
