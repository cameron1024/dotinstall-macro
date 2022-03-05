use syn::{parse::Parse, LitStr};

use super::kw;

pub struct Script {
    pub exec_kw: kw::exec,
    pub path: LitStr,
}

impl Parse for Script {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let exec_kw = input.parse()?;
        let path = input.parse()?;

        Ok(Script { exec_kw, path })
    }
}
