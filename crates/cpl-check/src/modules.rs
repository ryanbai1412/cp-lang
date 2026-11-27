//! Where the file behind `use std.segtree` or `use lib.graph.two_sat` is.
//!
//! A module is one `.cp` file (`plan/40-tooling.md`). `std` is `std`,
//! built into the compiler; any other first name is a
//! library: a folder the nearest `cp.toml` above the importing file names
//! under `[libs]`, or else a folder with a `lib.toml` that says
//! `name = "<that name>"`, found by walking up from the importing file.

use cpl_diag::{Diagnostic, Span};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
        load_library(&Disk, from, module)
    }
}

/// The library files a host with no file system gives by path, relative
/// to the file it checks (`cp.toml`, `kit/m.cp`): found as `FsLoader`
/// finds them on disk, through the same `cp.toml` and `lib.toml`.
pub struct GivenFiles(BTreeMap<String, String>);

impl GivenFiles {
    pub fn new(files: Vec<(String, String)>) -> GivenFiles {
        let mut given = BTreeMap::new();
        for (path, text) in files {
            given.insert(given_key(Path::new(&path)), text);
        }
        GivenFiles(given)
    }
}

impl Loader for GivenFiles {
    fn load(&self, from: &str, module: &[String]) -> Option<(String, String)> {
        load_library(self, from, module)
    }
}

/// Where libraries are read from: the disk, or a host's [`GivenFiles`].
trait Files {
    fn read(&self, path: &Path) -> Option<String>;
    fn is_dir(&self, path: &Path) -> bool;
    fn canonical(&self, path: &Path) -> PathBuf;
}

struct Disk;

impl Files for Disk {
    fn read(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }
    fn canonical(&self, path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }
}

/// `path` as a key of [`GivenFiles`]: its names joined by `/`, without `.`.
fn given_key(path: &Path) -> String {
    let path = path.to_string_lossy();
    let names = path
        .split(['/', '\\'])
        .filter(|name| !name.is_empty() && *name != ".");
    names.collect::<Vec<_>>().join("/")
}

impl Files for GivenFiles {
    fn read(&self, path: &Path) -> Option<String> {
        self.0.get(&given_key(path)).cloned()
    }
    fn is_dir(&self, path: &Path) -> bool {
        let dir = given_key(path);
        dir.is_empty()
            || self.0.keys().any(|f| {
                f.strip_prefix(&dir)
                    .is_some_and(|rest| rest.starts_with('/'))
            })
    }
    fn canonical(&self, path: &Path) -> PathBuf {
        PathBuf::from(given_key(path))
    }
}

/// The library module `module` imported by the file at `from`, from
/// `files`. Std is not a loader's: a check has its own (`GivenStd`).
fn load_library(files: &dyn Files, from: &str, module: &[String]) -> Option<(String, String)> {
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
        return None;
    }
    library_root(files, start, root_name).and_then(|root| {
        let path = root.join(&relative);
        let text = files.read(&path)?;
        Some((module_key(&path), text))
    })
}

/// A loaded file's path with `/` separators on every OS, so module lookups
/// that match on `/x.cp` also work on Windows.
fn module_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// The modules of `std` as they were when the compiler was built, by
/// their path below it (`"seq"`).
pub(crate) static BUNDLED: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/std_modules.rs"));

/// The modules of `std` as they are now on disk, so a std edit needs no
/// rebuild of `cpl`: in the `CPL_STD_DIR` folder when it is set, else, in a
/// dev build, in the std/ it was built from while that is still there.
/// Otherwise, and with `CPL_STD_DIR=` empty, those built in.
pub(crate) fn modules() -> &'static [(&'static str, &'static str)] {
    static FROM_DISK: std::sync::OnceLock<Option<Vec<(&'static str, &'static str)>>> =
        std::sync::OnceLock::new();
    FROM_DISK
        .get_or_init(|| {
            let dir = match cpl_env::STD_DIR.raw() {
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
    if built.is_empty() || cpl_env::STD_DIR.is_set() {
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
    let here_std = here.join(FROZEN_STD).canonicalize().ok()?;
    (built != here_std).then(|| {
        format!(
            "this dev `cpl` reads std from {}, but runs from the checkout at {}; {fix}",
            built.display(),
            here.display()
        )
    })
}

/// The std the old pipeline reads, pinned until stage 5 deletes it with this crate.
pub(crate) const FROZEN_STD: &str = "crates/cpl-check/frozen/std";

/// The cp-lang checkout `path` is in.
fn checkout(path: &Path) -> Option<std::path::PathBuf> {
    path.ancestors()
        .find(|d| d.join(FROZEN_STD).is_dir() && d.join("crates/cpl-check/build.rs").is_file())
        .map(Path::to_path_buf)
}

/// Whether `modules()` reads std from disk rather than the built-in copy.
pub fn std_from_disk() -> bool {
    !std::ptr::eq(modules(), BUNDLED)
}

fn from_disk(dir: &Path) -> Vec<(&'static str, &'static str)> {
    let mut found = Vec::new();
    crate::std_files::collect(dir, "", &mut found);
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

/// Whether this build can be given std modules: a wasm build without
/// `lazy-std` carries them all, and leaves the code that takes them out.
const GIVEN: bool = cfg!(any(not(target_arch = "wasm32"), feature = "lazy-std"));

fn built_in(name: &str) -> Option<&'static str> {
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
    static TEXT: &str = include!(concat!(env!("OUT_DIR"), "/std_calls.rs"));
    // Two lists, each sorted as written: `call_names` reads a set and
    // `call_table` writes the rest from a map. Sorting them here instead
    // put a copy of core's sort for `(&str, u32)` in cpl.wasm.
    static CALLS: std::sync::OnceLock<[Vec<(&'static str, u32)>; 2]> = std::sync::OnceLock::new();
    let calls = CALLS.get_or_init(|| {
        let (counts, rest) = TEXT.split_once('\n').unwrap_or((TEXT, ""));
        let names = crate::std_files::call_names(BUILT_INDEX.iter().map(|(_, text)| *text));
        let indexed = names
            .into_iter()
            .zip(counts.split(' '))
            .filter_map(|(name, count)| Some((name, count.parse().ok()?)));
        let unindexed = rest.lines().filter_map(|line| {
            let (name, count) = line.split_once(' ')?;
            Some((name, count.parse().ok()?))
        });
        [indexed.collect(), unindexed.collect()]
    });
    calls
        .iter()
        .find_map(|calls| {
            let at = calls
                .binary_search_by(|(known, _)| (*known).cmp(name))
                .ok()?;
            Some(calls[at].1)
        })
        .unwrap_or(0)
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

// `static`, not `const`: a const is copied into every crate that inlines a
// reader of it, and cpl.wasm carried std's index twice.
pub(crate) static BUILT_INDEX: &[(&str, &str)] =
    include!(concat!(env!("OUT_DIR"), "/std_index.rs"));

fn outlines(modules: &[(&'static str, &str)]) -> Vec<(&'static str, &'static str)> {
    let leak = |s: String| &*Box::leak(s.into_boxed_str());
    modules
        .iter()
        .map(|(name, text)| (*name, leak(crate::std_files::outline(text))))
        .collect()
}

fn in_index(name: &str) -> bool {
    outline_of(name).is_some()
}

/// The text of the std module `name` (`"geo/hull"`) the build carries.
pub fn bundled_std(name: &str) -> Option<&'static str> {
    built_in(name)
}

/// How much of std a check starts with, before what its host gave.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Carried {
    /// What the build carries: all of std, except in a browser build.
    #[default]
    Build,
    /// What a browser build carries (`std_files::EMBEDDED`), in any build.
    Embedded,
    Nothing,
}

/// The std a check reads: the modules the build carries and the ones its
/// host gave. A value, so two checks with different std do not meet.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct GivenStd {
    carried: Carried,
    /// By name, in the order of the names, each without its tests.
    modules: Vec<(String, std::sync::Arc<str>)>,
}

impl GivenStd {
    /// Std as a browser build has it before its page gives the rest, in
    /// any build: how a native test drives the giving.
    pub fn embedded() -> GivenStd {
        GivenStd {
            carried: Carried::Embedded,
            modules: Vec::new(),
        }
    }

    /// No module but those given.
    pub fn nothing() -> GivenStd {
        GivenStd {
            carried: Carried::Nothing,
            modules: Vec::new(),
        }
    }

    fn carried(&self, name: &str) -> Option<&'static str> {
        match self.carried {
            Carried::Build => built_in(name),
            Carried::Embedded if crate::std_files::EMBEDDED.contains(&name) => built_in(name),
            _ => None,
        }
    }

    fn given(&self, name: &str) -> Result<usize, usize> {
        self.modules
            .binary_search_by(|(known, _)| known.as_str().cmp(name))
    }

    /// The text of the module `name` (`"geo/hull"`), carried or given.
    pub fn text(&self, name: &str) -> Option<&str> {
        if let Some(text) = self.carried(name) {
            return Some(text);
        }
        if !GIVEN {
            return None;
        }
        self.given(name).ok().map(|at| &*self.modules[at].1)
    }

    /// Takes `text`, the file `std/<module>.cp` (`"sparse"`, `"geo/hull"`):
    /// how a browser build, which carries only `std_files::EMBEDDED`, gets
    /// the others. True when that changed what this has; a module the
    /// build carries, or one given again, is no change.
    pub fn give(self: &mut Arc<Self>, module: &str, text: &str) -> Result<bool, String> {
        if !in_index(module) {
            return Err(format!(
                "no module `std.{}` in the stdlib",
                module.replace('/', ".")
            ));
        }
        if !GIVEN || self.carried(module).is_some() {
            return Ok(false);
        }
        let text = crate::std_files::without_tests(text);
        let at = self.given(module);
        if at.is_ok_and(|at| *self.modules[at].1 == *text) {
            return Ok(false);
        }
        // Its own copy when a snapshot shares it.
        let modules = &mut Arc::make_mut(self).modules;
        match at {
            Ok(at) => modules[at].1 = text.into(),
            Err(at) => modules.insert(at, (module.to_string(), text.into())),
        }
        Ok(true)
    }

    /// The std module `parts` names when std has it and this lacks its
    /// text, with its outline.
    fn lacked(&self, parts: &[String]) -> Option<(String, &'static str)> {
        if !GIVEN {
            return None;
        }
        let (_, rest) = parts.split_first().filter(|(std, _)| *std == "std")?;
        // `use std.math.gcd` names `gcd` in `std.math`.
        for module in [rest, &rest[..rest.len().saturating_sub(1)]] {
            let name = module.join("/");
            if let Some(outline) = outline_of(&name).filter(|_| self.text(&name).is_none()) {
                return Some((name, outline));
            }
        }
        None
    }

    /// Whether `parts` names a std module the host has not given yet.
    pub(crate) fn lacks(&self, parts: &[String]) -> bool {
        self.lacked(parts).is_some()
    }

    /// What a check reads of a std module this lacks, `parts` or the one
    /// before its last name: its outline, path and how many of `parts`
    /// name it. Its signatures and doc lines answer hover and completion
    /// until the host gives the text.
    pub fn outline(&self, parts: &[String]) -> Option<(usize, String, &'static str)> {
        let (name, text) = self.lacked(parts)?;
        let len = name.split('/').count() + 1;
        Some((len, format!("std/{name}.cp"), text))
    }

    /// The std modules a check of `source`, the file `name`, loads that
    /// this lacks, by name (`"sparse"`): those its `use` lines and prelude
    /// names and `.method` calls bring in, and those they use in turn. It
    /// reads the outlines of the modules it lacks, so a module only their
    /// bodies use comes up once they are given: ask again after giving
    /// them, until it names none.
    pub fn needed(self: &Arc<Self>, name: &str, source: &str) -> Vec<String> {
        if !GIVEN {
            return Vec::new();
        }
        let mut db = crate::Database::new();
        db.std = self.clone();
        db.outlining = true;
        db.set_file(name, source);
        db.analyze(name);
        db.std_needs(name)
    }
}

/// The outline of the std module `name`.
pub(crate) fn outline_of(name: &str) -> Option<&'static str> {
    let at = std_index()
        .binary_search_by(|(known, _)| (*known).cmp(name))
        .ok()?;
    Some(std_index()[at].1)
}

/// The error for `use` of a module that isn't there, at `whole`;
/// `lacked` when std has it and the check was not given its text.
pub(crate) fn missing(parts: &[String], whole: Span, lacked: bool) -> Diagnostic {
    let dotted = parts.join(".");
    if lacked {
        return cpl_diag::msgs::check::STD_NOT_LOADED
            .error(whole, &[&dotted])
            .with_help(
                "the host gives it with `provide_std`; `std_needed` lists what a check needs",
            );
    }
    let near = nearest_std(parts);
    let help = match (parts[0].as_str(), &near) {
        (_, Some(m)) => format!("did you mean `{m}`?"),
        ("std", None) => "the standard library's modules are the files of `std/`".to_string(),
        (lib, None) => format!(
            "`{lib}` is a library: name its folder under `[libs]` in a `cp.toml` at or above this file (`{lib} = \"path\"`), or put this file below a folder whose `lib.toml` says `name = \"{lib}\"`"
        ),
    };
    let row = match parts[0] == "std" {
        true => &cpl_diag::msgs::check::NO_MODULE_NOT_STD,
        false => &cpl_diag::msgs::check::NO_MODULE,
    };
    let mut diag = row.at(&[&dotted]).error(whole).with_help(help);
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

/// The folder of the library `name`: the one a `cp.toml` above `start`
/// gives it under `[libs]`, or else the nearest folder above `start`
/// whose `lib.toml` names the library.
fn library_root(files: &dyn Files, start: &Path, name: &str) -> Option<PathBuf> {
    let start = files.canonical(start);
    let configured = cpl_config::find_map(&start, &mut |f| files.read(f), |dir, config| {
        let path = configured_library(config, name)?;
        let path = match path.strip_prefix("~/") {
            Some(below_home) => cpl_env::home_dir()?.join(below_home),
            None => dir.join(path),
        };
        files.is_dir(&path).then_some(path)
    });
    configured.or_else(|| library_manifest_root(files, &start, name))
}

/// Every library a `cp.toml` at or above `start` names under `[libs]`,
/// with its folder; a nearer `cp.toml` wins for a name.
pub fn configured_libraries(start: &Path) -> Vec<(String, PathBuf)> {
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    let read = &mut |f: &Path| std::fs::read_to_string(f).ok();
    cpl_config::find_map(&start, read, |dir, config| {
        for line in cpl_config::table(config, "libs") {
            let Some((name, _)) = line.pair else { continue };
            if out.iter().any(|(n, _)| n == name) {
                continue;
            }
            if let Some(root) = library_root(&Disk, dir, name) {
                out.push((name.to_string(), root));
            }
        }
        None::<()>
    });
    out
}

/// The path `[libs]` of a `cp.toml` gives the library `name`, written as
/// `name = "path"` or `name = { path = "path" }`.
fn configured_library<'a>(config: &'a str, name: &str) -> Option<&'a str> {
    for line in cpl_config::table(config, "libs") {
        let Some((key, value)) = line.pair else {
            continue;
        };
        if key != name {
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

fn library_manifest_root(files: &dyn Files, start: &Path, name: &str) -> Option<PathBuf> {
    start.ancestors().find_map(|dir| {
        let manifest = files.read(&dir.join("lib.toml"))?;
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
    fn given_files_find_a_library_as_the_disk_does() {
        let files = [
            ("./cp.toml", "[libs]\nkit = \"kit\"\n"),
            ("kit/m.cp", "pub fn g() -> Int = 1\n"),
            ("other/lib.toml", "name = \"other\"\n"),
            ("other/x.cp", "pub fn h() -> Int = 2\n"),
        ];
        let given = GivenFiles::new(
            files
                .iter()
                .map(|(p, t)| (p.to_string(), t.to_string()))
                .collect(),
        );
        let module = |m: &str| m.split('.').map(str::to_string).collect::<Vec<_>>();
        let found = given.load("main.cp", &module("kit.m"));
        assert_eq!(
            found,
            Some(("kit/m.cp".into(), "pub fn g() -> Int = 1\n".into()))
        );
        let found = given
            .load("other/y.cp", &module("other.x"))
            .map(|(path, _)| path);
        assert_eq!(
            found.as_deref(),
            Some("other/x.cp"),
            "a `lib.toml` names its folder"
        );
        assert_eq!(given.load("main.cp", &module("kit.missing")), None);
        assert_eq!(given.load("main.cp", &module("nope.m")), None);
    }

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
                let mut std = Arc::new(GivenStd::nothing());
                for module in &have {
                    std.give(module, built_in(module).unwrap()).unwrap();
                }
                let needed = std.needed(name, source);
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
        let names = crate::prelude::public_names(found.iter().copied());
        assert_eq!(
            names,
            [("f".into(), "x".into()), ("g".into(), "graph/y".into())]
        );
    }

    /// The names read off the outlines are the ones read off the files.
    #[test]
    fn std_names_read_off_the_outlines_match_the_files() {
        let files = crate::prelude::public_names(BUNDLED.iter().copied());
        let outlines = crate::prelude::public_names(BUILT_INDEX.iter().copied());
        assert!(!files.is_empty());
        assert_eq!(outlines, files);
    }

    /// Each module of `std/` as its file has it, by name.
    fn std_sources() -> Vec<(String, String)> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("frozen/std");
        let mut modules = Vec::new();
        crate::std_files::collect(&dir, "", &mut modules);
        modules.sort();
        modules
            .into_iter()
            .map(|(name, path)| (name, std::fs::read_to_string(path).unwrap()))
            .collect()
    }

    /// The counts keyed on the index's words read back as the ones the
    /// sources have, the names no index text has included.
    #[test]
    fn std_calls_reads_back_each_count_of_the_sources() {
        let sources = std_sources();
        let mut counts: std::collections::BTreeMap<String, u32> = Default::default();
        for (_, text) in &sources {
            for (at, _) in text.match_indices('.') {
                if at > 0 && text.as_bytes()[at - 1] == b'.' {
                    continue;
                }
                let name: String = text[at + 1..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_') {
                    *counts.entry(name).or_default() += 1;
                }
            }
        }
        assert!(counts.get("len").is_some_and(|n| *n > 100));
        for (name, count) in counts {
            let want = if count >= 2 { count } else { 0 };
            assert_eq!(std_calls(&name), want, "{name}");
        }
        assert_eq!(std_calls("no_such_name_anywhere"), 0);
    }

    /// The gzipped bytes of std in a browser build's `data` (its index and
    /// the `std_calls` table), as `gzip -9` gives them. A shrink-only cap:
    /// a std change over it fails here, at land time, rather than at the
    /// site's wasm budget for `data` (web/scripts/wasm-crate-budgets.json),
    /// which holds every crate's strings too. Make room by cutting what the
    /// browser embeds or std itself; after a cut, lower the cap with it.
    #[test]
    fn std_in_the_browser_build_has_no_comments_or_tests() {
        for (name, text) in std_sources() {
            let shipped = crate::std_bundle::browser_index(&name, &text);
            assert_eq!(
                shipped.lines().count(),
                text.lines().count(),
                "{name}: lines moved"
            );
            for (at, line) in shipped.lines().enumerate() {
                let line = line.trim_start();
                assert!(
                    !line.starts_with("//") && !line.starts_with("@test"),
                    "std.{name}:{} ships `{line}` to the browser",
                    at + 1
                );
            }
        }
    }

    /// The browser's index declares what the outline does: each line is
    /// the outline's, or its typed header with the `todo` body below moved
    /// onto it, and it parses as cleanly.
    #[test]
    fn std_in_the_browser_build_declares_what_its_outline_does() {
        for (name, text) in std_sources() {
            if crate::std_files::EMBEDDED.contains(&name.as_str()) {
                continue;
            }
            let text = crate::std_files::without_tests(&text);
            let outline = crate::std_bundle::without_private_fns(&crate::std_files::outline(&text));
            let shipped = crate::std_bundle::browser_index(&name, &text);
            let (was, now): (Vec<&str>, Vec<&str>) =
                (outline.lines().collect(), shipped.lines().collect());
            assert_eq!(was.len(), now.len(), "{name}: lines moved");
            for at in 0..was.len() {
                let moved = was[at].strip_suffix(':').is_some_and(|head| {
                    now[at] == format!("{head} = todo")
                        && was[at + 1].trim() == "todo"
                        && now[at + 1].is_empty()
                });
                let emptied = at > 0
                    && was[at].trim() == "todo"
                    && now[at].is_empty()
                    && now[at - 1].ends_with(" = todo");
                assert!(
                    moved || emptied || was[at] == now[at],
                    "std.{name}:{}: `{}` became `{}`",
                    at + 1,
                    was[at],
                    now[at]
                );
            }
            let errors = |t: &str| cpl_syntax::parse_file(t).diagnostics.len();
            assert_eq!(errors(&shipped), errors(&outline), "std.{name}");
        }
    }

    #[test]
    fn std_in_the_browser_build_is_within_its_gzipped_cap() {
        // Raised from 47,900 when lints moved out of cpl_lint into std's `@lint`
        // declarations, which the browser's linter reads from this data: their
        // 243 lines are 5,613 B gzipped, and cpl_lint's budget fell 4,700 B.
        const CAP: usize = 50_000;
        let sources = std_sources();
        let indexed: Vec<String> = sources
            .iter()
            .map(|(name, text)| crate::std_bundle::browser_index(name, text))
            .collect();
        let calls = crate::std_bundle::call_table(
            sources.iter().map(|(_, text)| text.as_str()),
            indexed.iter().map(String::as_str),
        );
        let mut data = indexed.concat();
        data.push_str(&calls);
        let mut gzip = std::process::Command::new("gzip")
            .args(["-9", "-c"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("gzip on PATH");
        let mut stdin = gzip.stdin.take().unwrap();
        let writer = std::thread::spawn(move || {
            std::io::Write::write_all(&mut stdin, data.as_bytes()).unwrap()
        });
        let out = gzip.wait_with_output().unwrap();
        writer.join().unwrap();
        let bytes = out.stdout.len();
        assert!(
            bytes <= CAP,
            "std in cpl.wasm is {bytes} B gzipped, over its cap of {CAP}: \
             cut what the browser build embeds (std_bundle::browser_index) or std itself"
        );
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
        std::fs::write(contest.join(cpl_config::FILE), "[libs]\nlib = \"../kit\"\n").unwrap();
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
        let found = library_root(&Disk, &root.join("contest"), "lib");
        if let Some(home) = home {
            std::env::set_var("HOME", home);
        }
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(found.map(|p| p.ends_with("kit")), Some(true));
    }
}

#[cfg(test)]
mod dev_std_tests {
    use super::{std_mismatch, FROZEN_STD};
    use std::path::{Path, PathBuf};

    fn checkout(root: &Path, name: &str) -> PathBuf {
        let dir = root.join(name);
        std::fs::create_dir_all(dir.join(FROZEN_STD)).unwrap();
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
        assert_eq!(
            std_mismatch(&a.join(FROZEN_STD), Some(&exe(&a)), None),
            None
        );
        let wrong = std_mismatch(&a.join(FROZEN_STD), Some(&exe(&b)), None).unwrap();
        assert!(wrong.contains("runs from the checkout at"), "{wrong}");
        let shared = root.join("shared/debug/cpl");
        assert!(std_mismatch(&a.join(FROZEN_STD), Some(&shared), Some(&b)).is_some());
        assert_eq!(
            std_mismatch(&a.join(FROZEN_STD), Some(&shared), Some(&root)),
            None
        );
        let gone = std_mismatch(&root.join("gone/std"), Some(&exe(&a)), None).unwrap();
        assert!(gone.contains("which is gone"), "{gone}");
        std::fs::remove_dir_all(&root).ok();
    }
}
