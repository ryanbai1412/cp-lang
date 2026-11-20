//! Where the file behind `use std.segtree` or `use lib.graph.two_sat` is.
//!
//! A module is one `.cp` file (`plan/40-tooling.md`). `std` is the
//! `stdlib/` folder shipped with the compiler; any other first name is a
//! library: a folder the nearest `cp.toml` above the importing file names
//! under `[libs]`, or else a folder with a `lib.toml` that says
//! `name = "<that name>"`, found by walking up from the importing file.

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

/// The folder of the library `name`: the one a `cp.toml` above `start`
/// gives it under `[libs]`, or else the nearest folder above `start`
/// whose `lib.toml` names the library.
fn library_root(start: &Path, name: &str) -> Option<PathBuf> {
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let configured = start.ancestors().find_map(|dir| {
        let config = std::fs::read_to_string(dir.join("cp.toml")).ok()?;
        let path = configured_library(&config, name)?;
        let path = match path.strip_prefix("~/") {
            Some(below_home) => PathBuf::from(std::env::var_os("HOME")?).join(below_home),
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
        assert_eq!(loaded.map(|(_, text)| text).as_deref(), Some("fn two() -> Int: 2\n"));
    }
}
