//! Where the file behind `use std.segtree` or `use lib.graph.two_sat` is.
//!
//! A module is one `.cp` file (`plan/40-tooling.md`). `std` is `std`,
//! built into the compiler; any other first name is a
//! library: a folder the nearest `cp.toml` above the importing file names
//! under `[libs]`, or else a folder with a `lib.toml` that says
//! `name = "<that name>"`, found by walking up from the importing file.

use cpl_diag::{codes, Diagnostic, Span};
use std::path::{Path, PathBuf};

/// Finds and reads modules. The language server and the tests replace the
/// file system with their own.
pub trait Loader: Send {
    /// The path and text of the module `module` (`["std", "segtree"]`),
    /// imported by the file at `from`. `None` when there is no such module.
    fn load(&self, from: &str, module: &[String]) -> Option<(String, String)>;
}

/// Reads modules from disk.
#[derive(Default)]
pub struct FsLoader;

impl Loader for FsLoader {
    fn load(&self, from: &str, module: &[String]) -> Option<(String, String)> {
        let (root_name, rest) = module.split_first()?;
        if rest.is_empty() {
            return None;
        }
        let from = file_path(from);
        let from = Path::new(&from);
        let start = from.parent().unwrap_or(Path::new("."));
        let start = if start.as_os_str().is_empty() {
            Path::new(".")
        } else {
            start
        };
        let mut relative = PathBuf::new();
        for part in rest {
            relative.push(part);
        }
        relative.set_extension("cp");
        if root_name == "std" {
            let text = bundled_std(rest)?;
            return Some((format!("std/{}.cp", rest.join("/")), text.to_string()));
        }
        library_root(start, root_name).and_then(|root| {
            let path = root.join(&relative);
            let text = std::fs::read_to_string(&path).ok()?;
            Some((path.to_string_lossy().into_owned(), text))
        })
    }
}

/// The modules of `std` as they were when the compiler was built, by
/// their path below it (`"seq"`).
const BUNDLED: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/std_modules.rs"));

/// The text of the standard library module `module` (`["segtree"]`) that
/// is built into the compiler.
/// The OS path a checker file name stands for: a `file://` URI decoded
/// (`%20` is a space; on Windows `/C:/x.cp` is `C:/x.cp`), any other name as is.
pub fn file_path(name: &str) -> String {
    match name.strip_prefix("file://") {
        Some(rest) => os_path(percent_decoded(rest), cfg!(windows)),
        None => name.to_string(),
    }
}

fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn os_path(path: String, windows: bool) -> String {
    let b = path.as_bytes();
    if windows && b.len() >= 3 && b[0] == b'/' && b[1].is_ascii_alphabetic() && b[2] == b':' {
        path[1..].to_string()
    } else {
        path
    }
}

pub fn bundled_std(module: &[String]) -> Option<&'static str> {
    let name = module.join("/");
    BUNDLED
        .binary_search_by(|(known, _)| (*known).cmp(name.as_str()))
        .ok()
        .map(|at| BUNDLED[at].1)
}

/// Every module of the standard library, by name (`"segtree"`), with its text.
pub fn std_modules() -> &'static [(&'static str, &'static str)] {
    BUNDLED
}

/// The error for `use` of a module that isn't there, at `whole`.
pub(crate) fn missing(parts: &[String], whole: Span) -> Diagnostic {
    let dotted = parts.join(".");
    let near = nearest_std(parts);
    let help = match (parts[0].as_str(), &near) {
        (_, Some(m)) => format!("did you mean `{m}`?"),
        ("std", None) => "the standard library's modules are the files of `std/`".to_string(),
        (lib, None) => format!(
            "`{lib}` is a library: name its folder under `[libs]` in a `cp.toml` at or above this file (`{lib} = \"path\"`), or put this file below a folder whose `lib.toml` says `name = \"{lib}\"`"
        ),
    };
    let message = match parts[0] == "std" {
        true => format!("no module `{dotted}`, and not in the stdlib (yet)"),
        false => format!("no module `{dotted}`"),
    };
    let mut diag = Diagnostic::error(codes::UNRESOLVED_MODULE, whole, message).with_help(help);
    if let Some(m) = near {
        diag.fix(format!("use `{m}`"), whole, m);
    }
    diag
}

/// `std.segtre` as `std.segtree`: the closest stdlib module to a missing one.
fn nearest_std(module: &[String]) -> Option<String> {
    let [std, name] = module else { return None };
    let names = BUNDLED.iter().map(|(known, _)| *known);
    (std == "std")
        .then(|| crate::check::closest(name, names))
        .flatten()
        .map(|m| format!("std.{m}"))
}

/// `HOME`, or `USERPROFILE` on Windows, where `HOME` is usually unset.
/// The one home lookup: libraries, setup, toolchains, crash reports, caches.
pub fn home_dir() -> Option<PathBuf> {
    let var = |name| std::env::var_os(name).filter(|v| !v.is_empty());
    var("HOME")
        .or_else(|| var("USERPROFILE"))
        .map(PathBuf::from)
}

/// The folder of the library `name`: the one a `cp.toml` above `start`
/// gives it under `[libs]`, or else the nearest folder above `start`
/// whose `lib.toml` names the library.
fn library_root(start: &Path, name: &str) -> Option<PathBuf> {
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let configured = start.ancestors().find_map(|dir| {
        let config = std::fs::read_to_string(dir.join("cp.toml")).ok()?;
        let path = configured_library(&config, name)?;
        let path = match path.strip_prefix("~/") {
            Some(below_home) => home_dir()?.join(below_home),
            None => dir.join(path),
        };
        path.is_dir().then_some(path)
    });
    configured.or_else(|| library_manifest_root(&start, name))
}

/// The path `[libs]` of a `cp.toml` gives the library `name`, written as
/// `name = "path"` or `name = { path = "path" }`.
fn configured_library<'a>(config: &'a str, name: &str) -> Option<&'a str> {
    let mut in_libs = false;
    for line in config.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            in_libs = line == "[libs]";
            continue;
        }
        if !in_libs {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != name {
            continue;
        }
        let value = value.trim();
        let value = match value.strip_prefix('{') {
            Some(table) => {
                let (_, path) = table.split_once("path")?;
                path.trim_start().strip_prefix('=')?.trim()
            }
            None => value,
        };
        return value.strip_prefix('"')?.split('"').next();
    }
    None
}

fn library_manifest_root(start: &Path, name: &str) -> Option<PathBuf> {
    start.ancestors().find_map(|dir| {
        let manifest = std::fs::read_to_string(dir.join("lib.toml")).ok()?;
        (library_name(&manifest)? == name).then(|| dir.to_path_buf())
    })
}

/// The `name = "..."` of a `lib.toml`.
fn library_name(manifest: &str) -> Option<&str> {
    manifest.lines().find_map(|line| {
        let value = line.trim().strip_prefix("name")?.trim_start();
        let value = value.strip_prefix('=')?.trim();
        value.strip_prefix('"')?.split('"').next()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uris_name_the_os_path() {
        assert_eq!(
            file_path("file:///home/a/my%20code/x.cp"),
            "/home/a/my code/x.cp"
        );
        assert_eq!(file_path("/work/x.cp"), "/work/x.cp");
        assert_eq!(os_path("/C:/Users/x.cp".into(), true), "C:/Users/x.cp");
        assert_eq!(os_path("/C:/Users/x.cp".into(), false), "/C:/Users/x.cp");
        assert_eq!(os_path("/home/x.cp".into(), true), "/home/x.cp");
    }

    #[test]
    fn a_library_is_named_by_its_manifest() {
        assert_eq!(library_name("# mine\nname = \"lib\"\n"), Some("lib"));
        assert_eq!(library_name("name=\"kit\" # short\n"), Some("kit"));
        assert_eq!(library_name("version = \"1\"\n"), None);
    }

    #[test]
    fn a_project_names_its_libraries_under_libs() {
        let config = "[target]\nlib = \"no\"\n[libs]\nlib = \"../kit\" # mine\ngeo = { path = \"~/geo\" }\n[new]\n";
        assert_eq!(configured_library(config, "lib"), Some("../kit"));
        assert_eq!(configured_library(config, "geo"), Some("~/geo"));
        assert_eq!(configured_library(config, "team"), None);
    }

    #[test]
    fn a_library_named_in_cp_toml_loads_from_another_folder() {
        let root = std::env::temp_dir().join(format!("cpl-libs-{}", std::process::id()));
        let contest = root.join("contest");
        let kit = root.join("kit/math");
        std::fs::create_dir_all(&contest).unwrap();
        std::fs::create_dir_all(&kit).unwrap();
        std::fs::write(contest.join("cp.toml"), "[libs]\nlib = \"../kit\"\n").unwrap();
        std::fs::write(kit.join("poly.cp"), "fn two() -> Int: 2\n").unwrap();
        let from = contest.join("a.cp");
        let module = ["lib", "math", "poly"].map(String::from);
        let loaded = FsLoader.load(&from.to_string_lossy(), &module);
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(
            loaded.map(|(_, text)| text).as_deref(),
            Some("fn two() -> Int: 2\n")
        );
    }

    #[test]
    fn an_editor_uri_with_a_space_finds_the_library_beside_it() {
        let root = std::env::temp_dir().join(format!("cpl uri {}", std::process::id()));
        std::fs::create_dir_all(root.join("contest")).unwrap();
        std::fs::create_dir_all(root.join("kit")).unwrap();
        std::fs::write(root.join("contest/cp.toml"), "[libs]\nlib = \"../kit\"\n").unwrap();
        std::fs::write(root.join("kit/poly.cp"), "fn two() -> Int: 2\n").unwrap();
        let path = root
            .join("contest/a.cp")
            .to_string_lossy()
            .replace('\\', "/");
        let slash = if path.starts_with('/') { "" } else { "/" };
        let uri = format!("file://{slash}{}", path.replace(' ', "%20"));
        let module = ["lib", "poly"].map(String::from);
        let loaded = FsLoader.load(&uri, &module);
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(
            loaded.map(|(_, text)| text).as_deref(),
            Some("fn two() -> Int: 2\n")
        );
    }

    #[test]
    fn a_home_library_path_uses_userprofile_without_home() {
        let root = std::env::temp_dir().join(format!("cpl home {}", std::process::id()));
        std::fs::create_dir_all(root.join("contest")).unwrap();
        std::fs::create_dir_all(root.join("kit")).unwrap();
        std::fs::write(root.join("contest/cp.toml"), "[libs]\nlib = \"~/kit\"\n").unwrap();
        let home = std::env::var_os("HOME");
        std::env::remove_var("HOME");
        std::env::set_var("USERPROFILE", &root);
        let found = library_root(&root.join("contest"), "lib");
        if let Some(home) = home {
            std::env::set_var("HOME", home);
        }
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(found.map(|p| p.ends_with("kit")), Some(true));
    }
}
