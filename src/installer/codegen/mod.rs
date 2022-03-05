use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;


use super::parse::{
    cargo::Cargo,
    ensure::Ensure,
    package::{Package, Packages},
    script::Script,
    symlinks::{Symlinks, Symlink},
    Installer, Section,
};

pub fn generate_installer(installer: &Installer) -> TokenStream {
    let install = installer.sections.iter().map(generate_section);

    quote! {
    
        use ::dotinstall::Installable;

        pub struct Installer {
            sections: ::std::vec::Vec<::std::boxed::Box<dyn ::dotinstall::Installable>>
        }

        impl ::dotinstall::Installable for Installer {
            fn install(&self, ctx: &::dotinstall::Context) -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error>> {
                for section in &self.sections {
                    section.install(ctx)?;
                }
                ::std::result::Result::Ok(())
            }
        }

        impl Installer {
            pub fn new() -> Self {
                let mut vec: ::std::vec::Vec<::std::boxed::Box<dyn ::dotinstall::Installable>> = ::std::vec![];

                #(#install)*

                Self { sections: vec }
            }
        }

    }
}

fn generate_section(section: &Section) -> TokenStream {
    match section {
        Section::Cargo(Cargo { crates, .. }) => {
            let crates = crates.iter().map(|c| quote! { #c.to_string() });
            quote! {
                let temp = ::dotinstall::CargoInstall { crates: ::std::vec![#(#crates),*] };
                vec.push(::std::boxed::Box::new(temp));
            }
        },
        Section::Ensure(Ensure {
            absolute_paths,
            home_paths,
            ..
        }) => {
            let absolute_paths = absolute_paths.iter().map(|p| quote! { #p.into() });
            let home_paths = home_paths.iter().map(|p| quote! { #p.into() });
            quote! {
                let absolute_paths = ::std::vec![#(#absolute_paths),*];
                let home_paths = ::std::vec![#(#home_paths),*];
                let temp = ::dotinstall::EnsureDirs { absolute_paths, home_paths };
                vec.push(::std::boxed::Box::new(temp));
            }
        },
        Section::Script(Script { path, .. }) => quote! {
            {
                let temp = ::dotinstall::Script { path: #path.into() };
                vec.push(::std::boxed::Box::new(temp));
            }
        },
        Section::Packages(Packages { packages, .. }) => {
            let mut build_vec = quote! {
                let mut packages = ::std::vec![];
            };

            for Package {
                name,
                pacman,
                apt,
                brew,
            } in packages
            {
                fn map_opt(o: &Option<LitStr>) -> TokenStream {
                    o.as_ref()
                        .map(|s| quote! { ::std::option::Option::Some(#s.to_string()) })
                        .unwrap_or(quote! {::std::option::Option::None})
                }

                let apt = map_opt(apt);
                let brew = map_opt(brew);
                let pacman = map_opt(pacman);

                build_vec.extend(quote! {
                   packages.push(::dotinstall::Package {
                      name: #name.to_string(),
                      pacman: #pacman,
                      apt: #apt,
                      brew: #brew,
                   });
                });
            }

            quote! {
                {
                    #build_vec
                    vec.push(::std::boxed::Box::new(packages));
                }
            }
        }
        Section::Symlinks(Symlinks { links, .. }) => {
            
            let links = links.iter().map(|Symlink { link, original, .. }| quote! {
                links.insert(#link.into(), #original.into());
            });

            quote! {
                {
                    let mut links = ::std::collections::HashMap::<::std::ffi::OsString, ::std::ffi::OsString>::new();
                    #(#links)*
                    let temp = ::dotinstall::Symlinks { links };
                    vec.push(::std::boxed::Box::new(temp));
                }

            }
        }
    }
}
