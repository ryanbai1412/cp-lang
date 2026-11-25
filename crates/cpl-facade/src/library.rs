//! The standard library a snapshot's files are checked against.

use crate::session::Session;

/// The standard library this compiler carries: its modules by name
/// (`"heap"`, `"geo/hull"`), their text, and the names they declare.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Std;

impl Std {
    /// The text of `module`, the file as it is in `std/`. None for a
    /// module the compiler knows only the outline of ([`Std::needed`]).
    pub fn text(self, module: &str) -> Option<&'static str> {
        cpl_check::bundled_std(module)
    }

    /// Every module by name with its outline: each top-level line, no bodies.
    pub fn outlines(self) -> &'static [(&'static str, &'static str)] {
        cpl_check::std_index()
    }

    /// The outline of the module `parts` names, or names a member of, when
    /// the compiler has not its text: how many of `parts` name the module,
    /// its path, and the outline.
    pub fn outline(self, parts: &[String]) -> Option<(usize, String, &'static str)> {
        cpl_check::std_outline(parts)
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
        cpl_check::std_methods().iter().map(|(path, name)| {
            let module: Vec<&str> = path[1..].iter().map(|part| part.text.as_str()).collect();
            (module.join("."), name.text.as_str())
        })
    }

    /// Whether a top-level fn of some module is named `name` and takes
    /// `self` on the line that names it.
    pub fn has_method(self, name: &str) -> bool {
        cpl_check::std_self_heads().contains_key(name)
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
            let given = have.clone();
            let more = cpl_check::std_needed_with(file, source, move |module| {
                (given.iter().any(|given| given == module))
                    .then(|| Std.text(module))
                    .flatten()
            });
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

    /// The modules a check of `source`, the file `file`, loads that the
    /// compiler has only the outline of: give each ([`Session::give_std`])
    /// and ask again, until it names none.
    pub fn needed(self, file: &str, source: &str) -> Vec<String> {
        cpl_check::std_needed(file, source)
    }

    /// The modules checks asked for and did not have since the last call.
    pub fn take_needs(self) -> Vec<String> {
        cpl_check::take_std_needs()
    }

    /// Whether the modules are read from a checkout's `std/` rather than
    /// the copy built into the compiler.
    pub fn from_disk(self) -> bool {
        cpl_check::std_from_disk()
    }
}

impl Session {
    /// The standard library this session's files are checked against.
    pub fn std(&self) -> Std {
        Std
    }

    /// Gives the compiler `text`, the file `std/<module>.cp`, for every
    /// query after. True when that changed what it has, and every answer
    /// that read the module's outline is then forgotten.
    pub fn give_std(&self, module: &str, text: &str) -> Result<bool, String> {
        let changed = cpl_check::provide_std(module, text)?;
        if changed {
            self.reload_modules();
        }
        Ok(changed)
    }
}
