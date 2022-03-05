use syn::{LitStr, parse::Parse, braced, punctuated::Punctuated, Token};

use super::kw;


pub struct Cargo {
    pub cargo_kw: kw::cargo,
    pub crates: Vec<LitStr>,
}

impl Parse for Cargo {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let cargo_kw = input.parse()?;
        let content;
        let _ = braced!(content in input);

        let crates = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
        let crates = crates.into_iter().collect();
        Ok(Self { cargo_kw, crates })
    }
}

#[cfg(test)]
mod test {
    use syn::parse_str;

    use super::*;

    #[test]
    fn parses_cargo_section() {
        let cargo: Cargo = parse_str(
            r#"cargo {
            "first",
            "second"
        }"#,
        )
        .unwrap();

        let crate_names: Vec<_> = cargo.crates.iter().map(|lit| lit.value()).collect();
        assert_eq!(crate_names, vec!["first", "second"]);
    }
}
