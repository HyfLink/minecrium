extern crate proc_macro;

use std::path::PathBuf;
use std::sync::Once;

use proc_macro::TokenStream;
use syn::{parse::Parse, Path};
use toml_edit::{Document, Item};

const MINECRIUM: &str = "minecrium";

/// Returns the cached manifest.
fn manifest() -> &'static Document {
    static mut MANIFEST: Option<Document> = None;
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        let manifest = match std::env::var_os("CARGO_MANIFEST_DIR") {
            Some(path) => {
                let mut path = PathBuf::from(path);
                path.push("Cargo.toml");
                let manifest = std::fs::read_to_string(path).unwrap();
                manifest.parse().unwrap()
            }
            None => panic!("'CARGO_MANIFEST_DIR' not found"),
        };

        unsafe { MANIFEST = Some(manifest) }
    });

    unsafe { MANIFEST.as_ref().unwrap() }
}

/// Returns the path for the crate with the given name.
///
/// See the crate `bevy_macro_utils`.
pub fn get_minecrium_path(name: &str) -> Path {
    fn parse_str<T: Parse>(path: &str) -> T {
        let tokens = path.parse::<TokenStream>().unwrap();
        syn::parse(tokens).unwrap()
    }

    fn dep_package(dep: &Item) -> Option<&str> {
        match dep.as_str() {
            Some(_) => None,
            None => match dep.get("package") {
                Some(name) => Some(name.as_str().unwrap()),
                None => None,
            },
        }
    }

    fn find_in_deps(name: &str, deps: &Item) -> Option<Path> {
        if let Some(dep) = deps.get(name) {
            let package = dep_package(dep).unwrap_or(name);
            let path = parse_str(package);
            Some(path)
        } else if let Some(dep) = deps.get(MINECRIUM) {
            let package = dep_package(dep).unwrap_or(MINECRIUM);
            let mut path: Path = parse_str(package);

            if let Some(module) = name.strip_prefix("minecrium_") {
                path.segments.push(parse_str(module));
            }

            Some(path)
        } else {
            None
        }
    }

    if let Some(deps) = manifest().get("dependencies") {
        if let Some(path) = find_in_deps(name, deps) {
            return path;
        }
    }

    if let Some(deps) = manifest().get("dev-dependencies") {
        if let Some(path) = find_in_deps(name, deps) {
            return path;
        }
    }

    return parse_str(name);
}
