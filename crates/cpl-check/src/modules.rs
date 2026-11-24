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
            Some((module_key(&path), text))
        })
    }
}

/// A loaded file's path with `/` separators on every OS, so module lookups
/// that match on `/x.cp` also work on Windows.
fn module_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// The modules of `std` as they were when the compiler was built, by
/// their path below it (`"seq"`).
const BUNDLED: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/std_modules.rs"));

/// The modules of `std` as they are now on disk, so a std edit needs no
/// rebuild of `cpl`: in the `CPL_STD_DIR` folder when it is set, else, in a
/// dev build, in the std/ it was built from while that is still there.
/// Otherwise, and with `CPL_STD_DIR=` empty, those built in.
pub(crate) fn modules() -> &'static [(&'static str, &'static str)] {
    static FROM_DISK: std::sync::OnceLock<Option<Vec<(&'static str, &'static str)>>> =
        std::sync::OnceLock::new();
    FROM_DISK
        .get_or_init(|| {
            let dir = match std::env::var_os("CPL_STD_DIR") {
                Some(dir) if dir.is_empty() => return None,
                Some(dir) => std::path::PathBuf::from(dir),
                None => source_std()?.to_path_buf(),
            };
            Some(from_disk(&dir)).filter(|found| !found.is_empty())
        })
        .as_deref()
        .unwrap_or(BUNDLED)
}

/// The std/ a dev build was compiled from (`build.rs`), if it is still a
/// folder; release and browser builds have none.
fn source_std() -> Option<&'static Path> {
    let dir = Path::new(env!("CPL_STD_SOURCE"));
    (!dir.as_os_str().is_empty() && dir.is_dir()).then_some(dir)
}

/// Why this dev `cpl` would check against the wrong std, if it would: its
/// std/ is gone, or it runs from another checkout than the one it was built
/// in (a copied or shared target dir), whose std it would silently ignore.
pub fn dev_std_problem() -> Option<String> {
    let built = env!("CPL_STD_SOURCE");
    if built.is_empty() || std::env::var_os("CPL_STD_DIR").is_some() {
        return None;
    }
    let exe = std::env::current_exe().ok();
    let cwd = std::env::current_dir().ok();
    std_mismatch(Path::new(built), exe.as_deref(), cwd.as_deref())
}

fn std_mismatch(built: &Path, exe: Option<&Path>, cwd: Option<&Path>) -> Option<String> {
    let fix =
        "rebuild it there (`touch crates/cpl-check/build.rs && cargo build`) or set CPL_STD_DIR";
    let Ok(built) = built.canonicalize() else {
        return Some(format!(
            "this dev `cpl` reads std from {}, which is gone; {fix}",
            built.display()
        ));
    };
    let here = exe.and_then(checkout).or_else(|| cwd.and_then(checkout))?;
    let here_std = here.join("std").canonicalize().ok()?;
    (built != here_std).then(|| {
        format!(
            "this dev `cpl` reads std from {}, but runs from the checkout at {}; {fix}",
            built.display(),
            here.display()
        )
    })
}

/// The cp-lang checkout `path` is in.
fn checkout(path: &Path) -> Option<std::path::PathBuf> {
    path.ancestors()
        .find(|d| d.join("std").is_dir() && d.join("crates/cpl-check/build.rs").is_file())
        .map(Path::to_path_buf)
}

/// Whether `modules()` reads std from disk rather than the built-in copy.
pub(crate) fn std_from_disk() -> bool {
    !std::ptr::eq(modules(), BUNDLED)
}

fn from_disk(dir: &Path) -> Vec<(&'static str, &'static str)> {
    let mut found = Vec::new();
    crate::std_files::collect(dir, dir, &mut found);
    found.sort();
    found
        .into_iter()
        .filter_map(|(name, path)| {
            let text = crate::std_files::without_tests(&std::fs::read_to_string(path).ok()?);
            Some((
                &*Box::leak(name.into_boxed_str()),
                &*Box::leak(text.into_boxed_str()),
            ))
        })
        .collect()
}

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
    modules()
        .binary_search_by(|(known, _)| (*known).cmp(name.as_str()))
        .ok()
        .map(|at| modules()[at].1)
}

/// Every module of the standard library, by name (`"segtree"`), with its text.
pub fn std_modules() -> &'static [(&'static str, &'static str)] {
    modules()
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
    let names = modules().iter().map(|(known, _)| *known);
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

/// Every library a `cp.toml` at or above `start` names under `[libs]`,
/// with its folder; a nearer `cp.toml` wins for a name.
pub fn configured_libraries(start: &Path) -> Vec<(String, PathBuf)> {
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    for dir in start.ancestors() {
        let Ok(config) = std::fs::read_to_string(dir.join("cp.toml")) else {
            continue;
        };
        let mut in_libs = false;
        for line in config.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.starts_with('[') {
                in_libs = line == "[libs]";
                continue;
            }
            let Some((name, _)) = line.split_once('=').filter(|_| in_libs) else {
                continue;
            };
            let name = name.trim();
            if out.iter().any(|(n, _)| n == name) {
                continue;
            }
            if let Some(root) = library_root(dir, name) {
                out.push((name.to_string(), root));
            }
        }
    }
    out
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
    fn std_dir_modules_are_read_from_disk_without_tests() {
        let dir = std::env::temp_dir().join(format!("cpl-std-dir-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("graph")).unwrap();
        std::fs::write(
            dir.join("x.cp"),
            "@test\nfn t():\n    pass\npub fn f() -> Int: 1\n",
        )
        .unwrap();
        std::fs::write(dir.join("graph/y.cp"), "pub fn g() -> Int: 2\n").unwrap();
        let found = from_disk(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            found,
            [
                ("graph/y", "pub fn g() -> Int: 2\n"),
                ("x", "\n\n\npub fn f() -> Int: 1\n")
            ]
        );
        let names = crate::std_files::public_names(found.iter().copied());
        assert_eq!(
            names,
            [("f".into(), "x".into()), ("g".into(), "graph/y".into())]
        );
    }

    /// The names a run from disk reads off the files are the ones the build
    /// put in, for the same std.
    #[test]
    fn std_names_read_at_run_time_match_the_built_in_table() {
        let read: Vec<(String, String)> = crate::std_files::public_names(BUNDLED.iter().copied());
        let table: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/std_names.rs"));
        let built: Vec<(String, String)> = table
            .iter()
            .map(|(n, m)| (n.to_string(), m.to_string()))
            .collect();
        assert_eq!(read, built);
    }

    #[test]
    fn a_library_file_key_uses_forward_slashes() {
        assert_eq!(
            module_key(Path::new(r"C:\kit\geo\poly.cp")),
            "C:/kit/geo/poly.cp"
        );
        assert_eq!(
            module_key(Path::new("/kit/geo/poly.cp")),
            "/kit/geo/poly.cp"
        );
    }

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

#[cfg(test)]
mod dev_std_tests {
    use super::std_mismatch;
    use std::path::{Path, PathBuf};

    fn checkout(root: &Path, name: &str) -> PathBuf {
        let dir = root.join(name);
        std::fs::create_dir_all(dir.join("std")).unwrap();
        std::fs::create_dir_all(dir.join("crates/cpl-check")).unwrap();
        std::fs::write(dir.join("crates/cpl-check/build.rs"), "").unwrap();
        std::fs::create_dir_all(dir.join("target/debug")).unwrap();
        dir
    }

    #[test]
    fn a_dev_cpl_names_the_std_of_another_checkout() {
        let root = std::env::temp_dir().join(format!("cpl-dev-std-{}", std::process::id()));
        let (a, b) = (checkout(&root, "a"), checkout(&root, "b"));
        let exe = |c: &Path| c.join("target/debug/cpl");
        assert_eq!(std_mismatch(&a.join("std"), Some(&exe(&a)), None), None);
        let wrong = std_mismatch(&a.join("std"), Some(&exe(&b)), None).unwrap();
        assert!(wrong.contains("runs from the checkout at"), "{wrong}");
        let shared = root.join("shared/debug/cpl");
        assert!(std_mismatch(&a.join("std"), Some(&shared), Some(&b)).is_some());
        assert_eq!(
            std_mismatch(&a.join("std"), Some(&shared), Some(&root)),
            None
        );
        let gone = std_mismatch(&root.join("gone/std"), Some(&exe(&a)), None).unwrap();
        assert!(gone.contains("which is gone"), "{gone}");
        std::fs::remove_dir_all(&root).ok();
    }
}
