//! The standard library a snapshot's files are checked against.

use crate::session::Session;

/// The standard library this compiler carries: its modules by name
/// (`"heap"`, `"geo/hull"`), their text, and the names they declare.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Std;

impl Std {
    /// The text of `module`, the file as it is in `std/`. None for a
    /// module this build does not carry ([`Session::std_needs`]).
    pub fn text(self, module: &str) -> Option<&'static str> {
        cpl_check::bundled_std(module)
    }

    /// Every module by name with its outline: each top-level line, no bodies.
    pub fn outlines(self) -> &'static [(&'static str, &'static str)] {
        cpl_check::std_index()
    }

    /// Every public name with its module, sorted by name.
    pub fn names(self) -> &'static [(&'static str, &'static str)] {
        cpl_check::std_names()
    }

    /// The module that declares the public name `name`.
    pub fn module_of(self, name: &str) -> Option<&'static str> {
        cpl_check::std_module(name)
    }

    /// The module and struct of each specialization the family name
    /// `name` stands for.
    pub fn family(self, name: &str) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        cpl_check::std_family(name)
    }

    /// Each `pub fn name(self: T, ..)` at the top of a module, as its
    /// module (`"geo.hull"`) and name: what `.name` reaches without a `use`.
    pub fn methods(self) -> impl Iterator<Item = (String, &'static str)> {
        cpl_check::std_methods().iter().map(|method| {
            let module: Vec<&str> = method.module[1..]
                .iter()
                .map(|part| part.text.as_str())
                .collect();
            (module.join("."), method.name.text.as_str())
        })
    }

    /// Whether some module gives a method named `name`.
    pub fn has_method(self, name: &str) -> bool {
        cpl_check::is_std_method(name)
    }

    /// How often std's own text calls a method named `name`.
    pub fn calls(self, name: &str) -> u32 {
        cpl_check::std_calls(name)
    }

    /// Every module a check of `source`, the file `file`, loads, those
    /// their bodies load in turn among them, sorted.
    pub fn used_by(self, file: &str, source: &str) -> Vec<String> {
        let mut have: Vec<String> = Vec::new();
        loop {
            let mut given = std::sync::Arc::new(cpl_check::GivenStd::nothing());
            for module in &have {
                let _ = given.give(module, Std.text(module).unwrap_or_default());
            }
            let more = given.needed(file, source);
            let more: Vec<String> = (more.into_iter())
                .filter(|module| !have.contains(module))
                .collect();
            if more.is_empty() {
                have.sort();
                return have;
            }
            have.extend(more);
        }
    }

    /// Whether the modules are read from a checkout's `std/` rather than
    /// the copy built into the compiler.
    /// Why this dev build would check against the wrong std, if it
    /// would: its std folder is gone, or is another checkout's.
    pub fn problem(self) -> Option<String> {
        cpl_check::dev_std_problem()
    }

    pub fn from_disk(self) -> bool {
        cpl_check::std_from_disk()
    }
}

impl Session {
    /// The standard library this session's files are checked against.
    pub fn std(&self) -> Std {
        Std
    }
}

/// Every library a `cp.toml` at or above `start` names under `[libs]`,
/// with its folder; a nearer `cp.toml` wins for a name.
pub fn libraries(start: &std::path::Path) -> Vec<(String, std::path::PathBuf)> {
    cpl_check::configured_libraries(start)
}

/// The OS path a file's name in a snapshot stands for: a `file://` URI
/// decoded, any other name as is.
pub fn file_path(name: &str) -> String {
    cpl_check::file_path(name)
}

/// The crates that answer behind the facade, in pipeline order.
#[cfg(feature = "cpp")]
pub const COMPONENTS: &[&str] = &[
    cpl_check::NAME,
    cpl_ir::NAME,
    cpl_codegen::NAME,
    cpl_interp::NAME,
];
