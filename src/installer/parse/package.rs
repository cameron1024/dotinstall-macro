use std::fmt::Debug;

use syn::{braced, parse::Parse, punctuated::Punctuated, spanned::Spanned, Error, LitStr, Token};

use super::kw;

pub struct Packages {
    pub packages_kw: kw::packages,
    pub packages: Vec<Package>,
}

impl Parse for Packages {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let packages_kw = input.parse()?;
        let contents;
        let _ = braced!(contents in input);

        let packages = Punctuated::<Package, Token![,]>::parse_terminated(&contents)?;
        let packages = packages.into_iter().collect();

        Ok(Self {
            packages,
            packages_kw,
        })
    }
}

pub struct Package {
    pub name: LitStr,
    pub pacman: Option<LitStr>,
    pub apt: Option<LitStr>,
    pub brew: Option<LitStr>,
}

impl Parse for Package {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let arrow: Option<Token![=>]> = input.parse()?;

        match arrow {
            None => Ok(Package {
                name,
                pacman: None,
                apt: None,
                brew: None,
            }),
            Some(_) => {
                let content;
                let _ = braced!(content in input);
                let overrides = Punctuated::<Override, Token![,]>::parse_terminated(&content)?;
                let overrides = Vec::from_iter(overrides);

                let apt = match single(overrides.iter(), |o| o.is_apt()) {
                    Ok(package) => Some(package.value.clone()),
                    Err(None) => None,
                    Err(Some(value)) => {
                        return Err(Error::new(value.apt.span(), "multiple apt overrides"))
                    }
                };

                let brew = match single(overrides.iter(), |o| o.is_brew()) {
                    Ok(package) => Some(package.value.clone()),
                    Err(None) => None,
                    Err(Some(value)) => {
                        return Err(Error::new(value.brew.span(), "multiple brew overrides"))
                    }
                };

                let pacman = match single(overrides.iter(), |o| o.is_pacman()) {
                    Ok(package) => Some(package.value.clone()),
                    Err(None) => None,
                    Err(Some(value)) => {
                        return Err(Error::new(value.pacman.span(), "multiple pacman overrides"))
                    }
                };

                Ok(Package {
                    name,
                    pacman,
                    apt,
                    brew,
                })
            }
        }
    }
}

/// Given an iterator and a predicate, returns Ok(value) if it contains exactly 1 element, Err(None) if it contains 0, and Err(Some(value)) if it contains 2 or more
fn single<I, T, F>(iter: I, mut pred: F) -> Result<T, Option<T>>
where
    I: Iterator<Item = T>,
    F: FnMut(&T) -> bool,
{
    let mut found = None;
    for t in iter {
        if pred(&t) {
            match found {
                Some(_) => return Err(Some(t)),
                None => found = Some(t),
            }
        }
    }
    found.ok_or(None)
}

struct Override {
    pacman: Option<kw::pacman>,
    apt: Option<kw::apt>,
    brew: Option<kw::brew>,
    value: LitStr,
}

impl Override {
    fn is_pacman(&self) -> bool {
        self.pacman.is_some()
    }
    fn is_apt(&self) -> bool {
        self.apt.is_some()
    }
    fn is_brew(&self) -> bool {
        self.brew.is_some()
    }
}

impl Debug for Override {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Override")
    }
}

impl Parse for Override {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut apt = None;
        let mut brew = None;
        let mut pacman = None;

        if input.peek(kw::apt) {
            apt = Some(input.parse()?);
        }
        if input.peek(kw::pacman) {
            pacman = Some(input.parse()?);
        }
        if input.peek(kw::brew) {
            brew = Some(input.parse()?);
        }

        input.parse::<Token![=]>()?;

        Ok(Self {
            value: input.parse()?,
            apt,
            brew,
            pacman,
        })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_str;

    use super::*;

    #[test]
    fn correctly_parses_overrides() {
        let o: Override = parse_str(r#"brew = "asdf""#).unwrap();
        assert!(
            matches!(o, Override { apt: None, brew: Some(_), pacman: None, value } if value.value() == "asdf")
        );

        let o: Override = parse_str(r#"pacman = "asdf""#).unwrap();
        assert!(
            matches!(o, Override { apt: None, brew: None, pacman: Some(_), value } if value.value() == "asdf")
        );

        let o: Override = parse_str(r#"apt = "asdf""#).unwrap();
        assert!(
            matches!(o, Override { apt: Some(_), brew: None, pacman: None, value } if value.value() == "asdf")
        );

        parse_str::<Override>(r#"asdf = "asdf""#).unwrap_err();
    }

    #[test]
    fn correctly_parses_package() {
        let p: Package = parse_str(
            r#"
            "some_package" => {
                apt = "apt_ov",
                brew = "brew_ov",
                pacman = "pacman_ov",
            }
            "#,
        )
        .unwrap();
        assert_eq!(p.name.value(), "some_package");
        assert_eq!(p.apt.unwrap().value(), "apt_ov");
        assert_eq!(p.brew.unwrap().value(), "brew_ov");
        assert_eq!(p.pacman.unwrap().value(), "pacman_ov");

        let empty_package: Package = parse_str(r#""some_package""#).unwrap();
        assert!(
            matches!(empty_package, Package { name, pacman: None, apt: None, brew: None} if name.value() == "some_package")
        );

        let err = parse_str::<Package>(r#""some_package" => { apt = "foo", apt = "foo" }"#);
        assert!(err.is_err());
    }

    #[test]
    fn correctly_parses_packages() {
        let p: Packages = parse_str(
            r#"
            packages {
                "some_package" => {
                    apt = "apt_ov",
                    brew = "brew_ov",
                    pacman = "pacman_ov",
                },
                "other_package"
            }
            "#,
        )
        .unwrap();

        let first = &p.packages[0];
        assert_eq!(first.name.value(), "some_package");
        assert_eq!(first.apt.as_ref().unwrap().value(), "apt_ov");
        assert_eq!(first.brew.as_ref().unwrap().value(), "brew_ov");
        assert_eq!(first.pacman.as_ref().unwrap().value(), "pacman_ov");

        let second = &p.packages[1];
        assert!(
            matches!(second, Package { name, pacman: None, apt: None, brew: None} if name.value() == "other_package")
        );
    }
}
