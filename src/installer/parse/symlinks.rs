use std::collections::HashSet;

use syn::{braced, parse::Parse, punctuated::Punctuated, Error, LitStr, Token};

use super::kw;

pub struct Symlinks {
    pub symlinks_kw: kw::symlinks,
    pub links: Vec<Symlink>,
}

impl Parse for Symlinks {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let symlinks_kw = input.parse()?;
        let content;
        let _ = braced!(content in input);

        let links = Punctuated::<Symlink, Token![,]>::parse_terminated(&content)?;

        let mut link_set = HashSet::new();

        for link in &links {
            if !link_set.insert(link.link.value()) {
                return Err(Error::new(
                    link.link.span(),
                    format!("Duplicate symlink for `{}`", link.link.value()),
                ));
            }
        }

        let links = links.into_iter().collect();

        Ok(Self { symlinks_kw, links })
    }
}

pub struct Symlink {
    pub original: LitStr,
    pub arrow: Token![=>],
    pub link: LitStr,
}

impl Parse for Symlink {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let link = input.parse()?;
        let arrow = input.parse()?;
        let original = input.parse()?;

        Ok(Self {
            original,
            arrow,
            link,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use syn::parse_str;

    use super::*;

    #[test]
    fn correctly_parses_symlinks() {
        let s: Symlinks = parse_str(
            r#"
            symlinks {
              "foo" => "bar",
              "~/.bashrc" => "~/config/bashrc",
            }
            "#,
        )
        .unwrap();

        let links: HashMap<_, _> = s
            .links
            .iter()
            .map(|l| (l.link.value(), l.original.value()))
            .collect();
        assert_eq!(links.get("foo"), Some(&"bar".to_string()));
        assert_eq!(links.get("~/.bashrc"), Some(&"~/config/bashrc".to_string()));
    }
}
