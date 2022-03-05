use syn::{LitStr, parse::Parse, braced, punctuated::Punctuated, Token};

use super::kw;


pub struct Ensure {
    pub ensure_kw: kw::ensure,
    pub absolute_paths: Vec<LitStr>,
    pub home_paths: Vec<LitStr>,
}

impl Parse for Ensure {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ensure_kw = input.parse()?;
        let content;
        let _ = braced!(content in input);

        let paths = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
        let (home_paths, absolute_paths) = paths
            .into_iter()
            .partition(|lit| lit.value().starts_with('~'));

        Ok(Self {
            ensure_kw,
            home_paths,
            absolute_paths,
        })
    }
}


#[cfg(test)]
mod tests {
    use syn::parse_str;

    use super::*;
    
    #[test]
    fn parses_ensure_section() {
        let ensure: Ensure = parse_str(
            r#"ensure {
            "first",
            "second",
            "~/third"
        }"#,
        )
        .unwrap();

        let home_paths: Vec<_> = ensure.home_paths.iter().map(|lit| lit.value()).collect();
        assert_eq!(home_paths, vec!["~/third"]);
        let absolute_paths: Vec<_> = ensure.absolute_paths.iter().map(|lit| lit.value()).collect();
        assert_eq!(absolute_paths, vec!["first", "second"]);
    }
}
