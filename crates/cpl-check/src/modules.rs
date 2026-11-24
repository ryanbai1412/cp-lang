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
pub fn std_from_disk() -> bool {
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
    have(&module.join("/"))
}

/// Whether this build can be given std modules: a wasm build without
/// `lazy-std` carries them all, and leaves the code that takes them out.
const GIVEN: bool = cfg!(any(not(target_arch = "wasm32"), feature = "lazy-std"));

fn built_in(name: &str) -> Option<&'static str> {
    if ONLY_EMBEDDED.load(std::sync::atomic::Ordering::Relaxed)
        && !crate::std_files::EMBEDDED.contains(&name)
    {
        return None;
    }
    modules()
        .binary_search_by(|(known, _)| (*known).cmp(name))
        .ok()
        .map(|at| modules()[at].1)
}

/// Every module of the standard library the compiler carries, by name
/// (`"segtree"`), with its text: all of them, except in a browser build,
/// which carries `std_files::EMBEDDED` and is given the rest.
#[cfg(test)]
pub(crate) fn std_modules() -> &'static [(&'static str, &'static str)] {
    modules()
}

/// How often std's own text writes `.name`, a method's name: how common
/// a member is, for ordering completion.
pub fn std_calls(name: &str) -> u32 {
    const CALLS: &[(&str, u32)] = include!(concat!(env!("OUT_DIR"), "/std_calls.rs"));
    CALLS
        .binary_search_by(|(known, _)| (*known).cmp(name))
        .map_or(0, |at| CALLS[at].1)
}

/// Every module of the standard library, by name, with its
/// `std_files::outline`: what the scans of std's declarations read, so
/// they need no module's whole text.
pub fn std_index() -> &'static [(&'static str, &'static str)] {
    static FROM_DISK: std::sync::OnceLock<Vec<(&'static str, &'static str)>> =
        std::sync::OnceLock::new();
    if cfg!(target_arch = "wasm32") || !std_from_disk() {
        return BUILT_INDEX;
    }
    FROM_DISK.get_or_init(|| outlines(modules()))
}

pub(crate) const BUILT_INDEX: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/std_index.rs"));

fn outlines(modules: &[(&'static str, &str)]) -> Vec<(&'static str, &'static str)> {
    let leak = |s: String| &*Box::leak(s.into_boxed_str());
    modules
        .iter()
        .map(|(name, text)| (*name, leak(crate::std_files::outline(text))))
        .collect()
}

fn in_index(name: &str) -> bool {
    std_index()
        .binary_search_by(|(known, _)| (*known).cmp(name))
        .is_ok()
}

static ONLY_EMBEDDED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Makes this native compiler carry only what a browser build does
/// (`std_files::EMBEDDED`), so a test can drive `provide_std` natively.
#[doc(hidden)]
pub fn carry_only_embedded_std() {
    ONLY_EMBEDDED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// The text of the std module `name` the compiler has, carried or given.
fn have(name: &str) -> Option<&'static str> {
    built_in(name).or_else(|| {
        let provided = provided();
        provided
            .iter()
            .find(|(known, _)| known == name)
            .map(|(_, text)| *text)
    })
}

/// The std modules a check of `source`, the file `name`, loads that the
/// compiler has not been given yet (`provide_std`), by name (`"sparse"`):
/// those its `use` lines and prelude names and `.method` calls bring in,
/// and those they use in turn. It reads the outlines of the modules it
/// lacks, so a module only their bodies use comes up once they are given:
/// ask again after giving them, until it names none.
pub fn std_needed(name: &str, source: &str) -> Vec<String> {
    std_needed_with(name, source, have)
}

/// `std_needed`, with `have` for the modules the compiler has.
#[doc(hidden)]
pub fn std_needed_with(
    name: &str,
    source: &str,
    have: impl Fn(&str) -> Option<&'static str> + Send + 'static,
) -> Vec<String> {
    if !GIVEN {
        return Vec::new();
    }
    let needed = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let loader = Outlining {
        have: Box::new(have),
        needed: needed.clone(),
    };
    let mut db = crate::Database::with_loader(Box::new(loader));
    db.set_file(name, source);
    db.analyze(name);
    let mut needed = needed.lock().unwrap_or_else(|e| e.into_inner()).clone();
    needed.sort();
    needed
}

/// The std text a compiler has of a module, by name.
type Has = dyn Fn(&str) -> Option<&'static str> + Send;

/// Loads what the compiler has of std, and the outline of what it lacks,
/// writing those down.
struct Outlining {
    have: Box<Has>,
    needed: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl Loader for Outlining {
    fn load(&self, from: &str, module: &[String]) -> Option<(String, String)> {
        let Some(("std", rest)) = module.split_first().map(|(s, r)| (s.as_str(), r)) else {
            return FsLoader.load(from, module);
        };
        let name = rest.join("/");
        let path = format!("std/{name}.cp");
        if let Some(text) = (self.have)(&name) {
            return Some((path, text.to_string()));
        }
        let at = std_index()
            .binary_search_by(|(known, _)| (*known).cmp(name.as_str()))
            .ok()?;
        add(
            &mut self.needed.lock().unwrap_or_else(|e| e.into_inner()),
            name,
        );
        Some((path, std_index()[at].1.to_string()))
    }
}

/// The std modules a host gave with `provide_std`, by name. A list, not a
/// map: the browser build has no other map from `String`.
type Provided = Vec<(String, &'static str)>;

static PROVIDED: std::sync::RwLock<Provided> = std::sync::RwLock::new(Vec::new());

/// The std modules a host gave with `provide_std`.
fn provided() -> std::sync::RwLockReadGuard<'static, Provided> {
    PROVIDED.read().unwrap_or_else(|e| e.into_inner())
}

/// Gives the compiler the text of the std module `module` (`"sparse"`,
/// `"geo/hull"`), the file as it is in `std/`: how a browser build, which
/// carries only `std_files::EMBEDDED`, gets the others. True when that
/// changed what the compiler has; giving a module again is no change.
pub fn provide_std(module: &str, text: &str) -> Result<bool, String> {
    if !in_index(module) {
        return Err(format!(
            "no module `std.{}` in the stdlib",
            module.replace('/', ".")
        ));
    }
    if !GIVEN || built_in(module).is_some() {
        return Ok(false);
    }
    let text = crate::std_files::without_tests(text);
    let mut provided = PROVIDED.write().unwrap_or_else(|e| e.into_inner());
    let known = provided.iter().position(|(known, _)| known == module);
    if known.is_some_and(|at| provided[at].1 == text) {
        return Ok(false);
    }
    let text: &'static str = Box::leak(text.into_boxed_str());
    match known {
        Some(at) => provided[at].1 = text,
        None => provided.push((module.to_string(), text)),
    }
    Ok(true)
}

/// The std modules a check asked for and did not have since the last
/// `take_std_needs`, by name (`"sparse"`).
static NEEDS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// The std modules checks needed and did not have since the last call.
pub fn take_std_needs() -> Vec<String> {
    if !GIVEN {
        return Vec::new();
    }
    let mut needs = std::mem::take(&mut *NEEDS.lock().unwrap_or_else(|e| e.into_inner()));
    needs.sort();
    needs
}

/// Adds `name` to the set `names`.
fn add(names: &mut Vec<String>, name: String) {
    if !names.contains(&name) {
        names.push(name);
    }
}

/// The std module `parts` names when the compiler knows it but does not
/// have its text yet.
fn unprovided(parts: &[String]) -> Option<String> {
    if !GIVEN {
        return None;
    }
    let (_, rest) = parts.split_first().filter(|(std, _)| *std == "std")?;
    // `use std.math.gcd` names `gcd` in `std.math`.
    [rest, &rest[..rest.len().saturating_sub(1)]]
        .into_iter()
        .find(|module| in_index(&module.join("/")) && bundled_std(module).is_none())
        .map(|module| module.join("/"))
}

/// What a check reads of a std module the compiler knows and has not been
/// given, `parts` or the one before its last name: its outline, path and
/// how many of `parts` name it. Its signatures and doc lines answer hover
/// and completion until the host gives the text.
pub fn std_outline(parts: &[String]) -> Option<(usize, String, &'static str)> {
    let name = unprovided(parts)?;
    let at = std_index()
        .binary_search_by(|(known, _)| (*known).cmp(name.as_str()))
        .ok()?;
    let len = name.split('/').count() + 1;
    Some((len, format!("std/{name}.cp"), std_index()[at].1))
}

/// The error for `use` of a module that isn't there, at `whole`.
pub(crate) fn missing(parts: &[String], whole: Span) -> Diagnostic {
    let dotted = parts.join(".");
    if let Some(name) = unprovided(parts) {
        add(&mut NEEDS.lock().unwrap_or_else(|e| e.into_inner()), name);
        return Diagnostic::error(
            codes::UNRESOLVED_MODULE,
            whole,
            format!("std module `{dotted}` is not loaded yet"),
        )
        .with_help("the host gives it with `provide_std`; `std_needed` lists what a check needs");
    }
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
    let names = std_index().iter().map(|(known, _)| *known);
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

    /// Asking `std_needed` until it names nothing gives the std modules a
    /// check with all of std loads: for a `use` of each module, and for the
    /// test programs, which also reach std through prelude names and
    /// `.method` calls.
    #[test]
    fn std_needed_asked_until_empty_names_what_a_check_loads() {
        type Seen = std::sync::Arc<std::sync::Mutex<std::collections::BTreeSet<String>>>;
        struct Recording(Seen);
        impl Loader for Recording {
            fn load(&self, from: &str, module: &[String]) -> Option<(String, String)> {
                if let Some(("std", rest)) = module.split_first().map(|(s, r)| (s.as_str(), r)) {
                    // `use std.rand.Rng` asks for a module `rand/Rng` first.
                    if in_index(&rest.join("/")) {
                        self.0.lock().unwrap().insert(rest.join("/"));
                    }
                }
                FsLoader.load(from, module)
            }
        }
        let programs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../cpl-cli/tests/programs");
        let mut sources: Vec<(String, String)> = std_index()
            .iter()
            .map(|(m, _)| {
                (
                    format!("use_{m}.cp"),
                    format!("use std.{}\n", m.replace('/', ".")),
                )
            })
            .collect();
        for entry in std::fs::read_dir(programs).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "cp") {
                let text = std::fs::read_to_string(&path).unwrap();
                sources.push((path.to_string_lossy().into_owned(), text));
            }
        }
        let embedded = || crate::std_files::EMBEDDED.iter().map(|m| m.to_string());
        let mut most = 0;
        for (name, source) in &sources {
            let seen = Seen::default();
            let mut db = crate::Database::with_loader(Box::new(Recording(seen.clone())));
            db.set_file(name, source.as_str());
            db.diagnostics(name);
            let mut loaded = seen.lock().unwrap().clone();
            loaded.extend(embedded());
            let mut have: std::collections::BTreeSet<String> = embedded().collect();
            let mut asked = 0;
            loop {
                let given = have.clone();
                let needed = std_needed_with(name, source, move |m| {
                    given.contains(m).then(|| built_in(m)).flatten()
                });
                if needed.is_empty() {
                    break;
                }
                have.extend(needed);
                asked += 1;
            }
            most = most.max(asked);
            assert_eq!(have, loaded, "{name}");
        }
        eprintln!("std_needed: at most {most} rounds");
    }

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
