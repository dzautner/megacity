//! Auto-discover modules from `.rs` files **and** directories containing `mod.rs`.
//!
//! Unlike the `automod` crate, this macro also discovers directory-based modules
//! (directories that contain a `mod.rs` file), making it suitable for projects
//! that mix single-file and directory-based modules.
//!
//! # Usage
//!
//! ```ignore
//! // Discover all modules in src/
//! automod_dir::dir!(pub "src");
//!
//! // Discover all modules except specific ones (for cfg-gated or private modules)
//! automod_dir::dir!(pub "src" exclude "plugin_registration", "test_harness", "integration_tests");
//! ```
//!
//! The path is relative to the crate's `Cargo.toml` directory (`CARGO_MANIFEST_DIR`).
//!
//! The macro generates `mod` (or `pub mod`) items for every discovered module,
//! sorted alphabetically for deterministic output.

#![allow(clippy::needless_pass_by_value)]

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token, Visibility};

struct Arg {
    vis: Visibility,
    path: LitStr,
    excludes: HashSet<String>,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        let path: LitStr = input.parse()?;
        let mut excludes = HashSet::new();

        // Parse optional: exclude "name1", "name2", ...
        if input.peek(Ident) {
            let kw: Ident = input.parse()?;
            if kw != "exclude" {
                return Err(syn::Error::new(kw.span(), "expected `exclude`"));
            }
            loop {
                let name: LitStr = input.parse()?;
                excludes.insert(name.value());
                if !input.peek(Token![,]) {
                    break;
                }
                let _comma: Token![,] = input.parse()?;
                // Allow trailing comma
                if input.is_empty() {
                    break;
                }
            }
        }

        Ok(Arg {
            vis,
            path,
            excludes,
        })
    }
}

/// Pull in every source file and directory-module in a directory.
///
/// Discovers:
/// - `*.rs` files (excluding `mod.rs`, `lib.rs`, `main.rs`)
/// - Directories that contain a `mod.rs` file
///
/// Each discovered item becomes a `mod` declaration. An optional visibility
/// prefix (e.g. `pub`) is applied to all generated items.
///
/// Use the `exclude` keyword to skip specific modules that need manual
/// declarations (e.g. cfg-gated or private modules):
///
/// ```ignore
/// automod_dir::dir!(pub "src" exclude "test_harness", "integration_tests");
/// ```
#[proc_macro]
pub fn dir(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Arg);
    let vis = &input.vis;
    let rel_path = input.path.value();

    let dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(manifest_dir) => PathBuf::from(manifest_dir).join(&rel_path),
        None => PathBuf::from(&rel_path),
    };

    let expanded = match discover_modules(&dir, &input.excludes) {
        Ok(names) => names
            .into_iter()
            .map(|name| {
                let ident = Ident::new(&name, Span::call_site());
                quote! { #vis mod #ident; }
            })
            .collect::<TokenStream2>(),
        Err(err) => syn::Error::new(Span::call_site(), err).to_compile_error(),
    };

    TokenStream::from(expanded)
}

fn discover_modules(
    dir: &std::path::Path,
    excludes: &HashSet<String>,
) -> Result<Vec<String>, String> {
    let mut names = Vec::new();

    let entries =
        fs::read_dir(dir).map_err(|e| format!("failed to read directory {:?}: {}", dir, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let metadata = fs::metadata(entry.path())
            .map_err(|e| format!("failed to read metadata for {:?}: {}", entry.path(), e))?;
        let file_name = entry.file_name();

        if metadata.is_file() {
            // Skip mod.rs, lib.rs, main.rs
            if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "main.rs" {
                continue;
            }
            let path = std::path::Path::new(&file_name);
            if path.extension() == Some(OsStr::new("rs")) {
                if let Ok(mut utf8) = file_name.into_string() {
                    utf8.truncate(utf8.len() - ".rs".len());
                    if !excludes.contains(&utf8) {
                        names.push(utf8);
                    }
                }
            }
        } else if metadata.is_dir() {
            // Include directory only if it contains a mod.rs
            let mod_rs = entry.path().join("mod.rs");
            if mod_rs.exists() {
                if let Ok(utf8) = file_name.into_string() {
                    if !excludes.contains(&utf8) {
                        names.push(utf8);
                    }
                }
            }
        }
    }

    if names.is_empty() {
        return Err(format!(
            "no source files or directory modules found in {:?}",
            dir
        ));
    }

    names.sort();
    Ok(names)
}
