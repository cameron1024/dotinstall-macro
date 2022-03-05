use syn::{parse::Parse, punctuated::Punctuated, Error, Token};

use self::{cargo::Cargo, ensure::Ensure, package::Packages, script::Script, symlinks::Symlinks};

pub mod cargo;
pub mod ensure;
pub mod package;
pub mod script;
pub mod symlinks;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(cargo);
    custom_keyword!(ensure);
    custom_keyword!(packages);
    custom_keyword!(symlinks);
    custom_keyword!(pacman);
    custom_keyword!(apt);
    custom_keyword!(brew);
    custom_keyword!(exec);
}

pub struct Installer {
    pub sections: Vec<Section>,
}

impl Parse for Installer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sections = Punctuated::<Section, Token![;]>::parse_terminated(input)?;
        let sections = sections.into_iter().collect();
        Ok(Self { sections })
    }
}

pub enum Section {
    Cargo(Cargo),
    Ensure(Ensure),
    Packages(Packages),
    Script(Script),
    Symlinks(Symlinks),
}

#[cfg(test)]
impl Section {
    fn as_cargo(&self) -> Option<&Cargo> {
        match self {
            Self::Cargo(c) => Some(c),
            _ => None,
        }
    }
    
    fn as_ensure(&self) -> Option<&Ensure> {
        match self {
            Self::Ensure(c) => Some(c),
            _ => None,
        }
    }

    fn as_packages(&self) -> Option<&Packages> {
        match self {
            Self::Packages(c) => Some(c),
            _ => None,
        }
    }

    fn as_script(&self) -> Option<&Script> {
        match self {
            Self::Script(c) => Some(c),
            _ => None,
        }
    }

    fn as_symlinks(&self) -> Option<&Symlinks> {
        match self {
            Self::Symlinks(c) => Some(c),
            _ => None,
        }
    }
}

impl Parse for Section {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::cargo) {
            Ok(Self::Cargo(input.parse()?))
        } else if input.peek(kw::ensure) {
            Ok(Self::Ensure(input.parse()?))
        } else if input.peek(kw::exec) {
            Ok(Self::Script(input.parse()?))
        } else if input.peek(kw::packages) {
            Ok(Self::Packages(input.parse()?))
        } else if input.peek(kw::symlinks) {
            Ok(Self::Symlinks(input.parse()?))
        } else {
            Err(Error::new(input.span(), "Unknown section"))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use syn::parse_str;

    use crate::installer::parse::package::Package;

    use super::*;

    #[test]
    fn full_parse_example() {
        let mut installer: Installer = parse_str(
            r#"

        cargo {
            "ripgrep",
            "exa"
        };

        ensure {
            "~/.local/bin",
            "/foo/bar",
        };

        exec "./install_fonts.sh";

        packages {
            "unzip",
            "build-essential" => {
              pacman = "base-devel",
            }
        };

        symlinks {
            "foo" => "bar",
        };
            "#,
        )
        .unwrap();

        let cargo = installer.sections.remove(0);
        assert_eq!(cargo.as_cargo().unwrap().crates[0].value(), "ripgrep");
        assert_eq!(cargo.as_cargo().unwrap().crates[1].value(), "exa");

        let ensure = installer.sections.remove(0);
        let ensure = ensure.as_ensure().unwrap();
        let home_paths = ensure
            .home_paths
            .iter()
            .map(|s| s.value())
            .collect::<Vec<_>>();
        assert_eq!(home_paths, ["~/.local/bin"]);
        let absolute_paths = ensure
            .absolute_paths
            .iter()
            .map(|s| s.value())
            .collect::<Vec<_>>();
        assert_eq!(absolute_paths, ["/foo/bar"]);

        let script = installer.sections.remove(0);
        assert_eq!(
            script.as_script().unwrap().path.value(),
            "./install_fonts.sh"
        );

        let packages = installer.sections.remove(0);
        let packages = packages.as_packages().unwrap();

        let unzip = &packages.packages[0];
        assert!(
            matches!(unzip, Package { name, pacman: None, apt: None, brew: None  } if name.value() == "unzip" )
        );

        let Package { name, pacman, apt, brew } = &packages.packages[1];
        assert_eq!(name.value(), "build-essential");
        assert_eq!(pacman.as_ref().unwrap().value(), "base-devel");
        assert!(apt.is_none());
        assert!(brew.is_none());
    }
}
