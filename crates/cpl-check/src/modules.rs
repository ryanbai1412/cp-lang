//! Where the file behind `use std.segtree` or `use lib.graph.two_sat` is.
//!
//! A module is one `.cp` file (`plan/40-tooling.md`). `std` is the
//! `stdlib/` folder shipped with the compiler; any other first name is a
//! library: a folder with a `lib.toml` that says `name = "<that name>"`,
//! found by walking up from the importing file.

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
        let from = Path::new(from.strip_prefix("file://").unwrap_or(from));
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
        let roots = if root_name == "std" {
            std_roots(start)
        } else {
            library_root(start, root_name).into_iter().collect()
        };
        let on_disk = roots.iter().find_map(|root| {
            let path = root.join(&relative);
            let text = std::fs::read_to_string(&path).ok()?;
            Some((path.to_string_lossy().into_owned(), text))
        });
        // No `stdlib/` to read (a browser, a binary on its own): the copy
        // the compiler was built with.
        on_disk.or_else(|| {
            let text = bundled_std(rest).filter(|_| root_name == "std")?;
            Some((format!("stdlib/{}.cp", rest.join("/")), text.to_string()))
        })
    }
}

/// The modules of `stdlib/` as they were when the compiler was built, by
/// their path below it (`"seq"`).
const BUNDLED_STD: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/std_modules.rs"));

/// The text of the standard library module `module` (`["segtree"]`) that
/// is built into the compiler.
pub fn bundled_std(module: &[String]) -> Option<&'static str> {
    let name = module.join("/");
    BUNDLED_STD
        .binary_search_by(|(known, _)| (*known).cmp(name.as_str()))
        .ok()
        .map(|at| BUNDLED_STD[at].1)
}

/// Folders that may hold the standard library, most specific first:
/// `$CPL_STD`, a `stdlib/` above the importing file (a checkout of the
/// compiler), and `stdlib/` next to or above the `cpl` binary.
fn std_roots(start: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(dir) = std::env::var_os("CPL_STD") {
        roots.push(PathBuf::from(dir));
    }
    let above = |from: &Path, roots: &mut Vec<PathBuf>| {
        let from = from.canonicalize().unwrap_or_else(|_| from.to_path_buf());
        for dir in from.ancestors() {
            let candidate = dir.join("stdlib");
            if candidate.is_dir() {
                roots.push(candidate);
            }
        }
    };
    above(start, &mut roots);
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            above(dir, &mut roots);
        }
    }
    roots
}

/// The nearest folder above `start` whose `lib.toml` names the library.
fn library_root(start: &Path, name: &str) -> Option<PathBuf> {
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
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
    fn a_library_is_named_by_its_manifest() {
        assert_eq!(library_name("# mine\nname = \"lib\"\n"), Some("lib"));
        assert_eq!(library_name("name=\"kit\" # short\n"), Some("kit"));
        assert_eq!(library_name("version = \"1\"\n"), None);
    }
}
