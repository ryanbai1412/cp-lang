//! The one way a tool gets checker answers (facts-session-api.md). A
//! [`Session`] owns the incremental [`Database`]; every edit moves it to a
//! new [`Revision`], and an analysis, its facts and its diagnostics are
//! built at most once per revision of a file. No crate but this one can
//! build a `Database`, so no tool keeps a second front end that could
//! disagree with the checker.

use crate::db::{Analysis, Database};
use crate::modules::Loader;
use crate::Facts;
use cpl_diag::Diagnostic;
use std::sync::Arc;

/// How many edits a session has seen. Answers built at one revision stay
/// true for that revision.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Revision(pub(crate) u64);

/// Where the modules of `std` come from.
#[derive(Default)]
pub enum StdSource {
    /// The std this compiler carries, or the one `CPL_STD_DIR` names.
    #[default]
    Bundled,
    /// Module texts by their path below `std/` (`"seq"`), handed in by a
    /// host with no file system: the browser, the judge sandbox.
    Given(Vec<(String, String)>),
}

/// Where `use lib.x` modules come from.
#[derive(Default)]
pub enum Libraries {
    /// The folders a `cp.toml` `[libs]` table (or a `lib.toml`) above the
    /// importing file names, read from disk.
    #[default]
    Discover,
    /// A host's own resolver (the language server's open buffers, tests).
    Loader(Box<dyn Loader>),
}

/// How a session finds modules. Every tool (CLI, LSP, wasm, judge) builds
/// its session from these, so all of them resolve `use` the same way.
#[derive(Default)]
pub struct SessionOptions {
    pub std: StdSource,
    pub libs: Libraries,
    /// The language edition files are read in. Shape only: one edition
    /// exists, so no value changes behaviour yet.
    pub edition: Option<String>,
    /// The version of the package being checked, for versioned libraries.
    /// Shape only, like `edition`.
    pub package_version: Option<String>,
}

pub struct Session {
    db: Database,
    edition: Option<String>,
    package_version: Option<String>,
}

impl Default for Session {
    fn default() -> Session {
        Session::new()
    }
}

/// The analysis of `text` as the file `path`, from a session of its own:
/// for a tool that checks one file once.
pub fn open(path: &str, text: impl Into<Arc<str>>) -> Arc<Analysis> {
    let mut session = Session::new();
    session.set_file(path, text);
    session.analysis(path).unwrap_or_default()
}

/// The analysis of an empty file.
impl Default for Analysis {
    fn default() -> Analysis {
        Analysis {
            text: Arc::from(""),
            lines: cpl_diag::LineIndex::new(""),
            has_syntax_errors: false,
            diagnostics: Vec::new(),
            items: Vec::new(),
            syntax: Arc::default(),
            scope: Arc::default(),
            facts: std::sync::OnceLock::new(),
            revision: Revision::default(),
        }
    }
}

impl Session {
    /// A session with the default options: bundled std, libraries found on disk.
    pub fn new() -> Session {
        Session::with_options(SessionOptions::default())
    }

    pub fn with_options(opts: SessionOptions) -> Session {
        if let StdSource::Given(modules) = &opts.std {
            for (module, text) in modules {
                // A module the build carries already, or a name std lacks, keeps the build's.
                let _ = crate::provide_std(module, text);
            }
        }
        let db = match opts.libs {
            Libraries::Discover => Database::new(),
            Libraries::Loader(loader) => Database::with_loader(loader),
        };
        Session {
            db,
            edition: opts.edition,
            package_version: opts.package_version,
        }
    }

    /// A session that asks `loader` for the modules its files import.
    pub fn with_loader(loader: Box<dyn Loader>) -> Session {
        Session::with_options(SessionOptions {
            libs: Libraries::Loader(loader),
            ..SessionOptions::default()
        })
    }

    pub fn edition(&self) -> Option<&str> {
        self.edition.as_deref()
    }

    pub fn package_version(&self) -> Option<&str> {
        self.package_version.as_deref()
    }

    /// Sets the text of the file at `path`; the session moves to a new revision.
    pub fn set_file(&mut self, path: &str, text: impl Into<Arc<str>>) -> Revision {
        self.db.set_file(path, text);
        self.revision()
    }

    pub fn remove_file(&mut self, path: &str) -> Revision {
        self.db.remove_file(path);
        self.revision()
    }

    /// The revision the last edit moved the session to.
    pub fn revision(&self) -> Revision {
        Revision(self.db.edits)
    }

    /// The file's analysis; asking again before an edit returns the same one.
    pub fn analysis(&mut self, path: &str) -> Option<Arc<Analysis>> {
        self.db.analyze(path)
    }

    /// What checking the file found about each place, at the analysis's revision.
    pub fn facts(&mut self, path: &str) -> Option<Arc<Facts>> {
        self.analysis(path).map(|a| a.facts().clone())
    }

    /// What checking found in the item of `path` around `at`. A module read
    /// through `use` has only its signatures checked; this checks the one
    /// body asked for (memoised per item), so lowering a std fn reads the
    /// checker's answers for it too.
    pub fn body_facts(&mut self, path: &str, at: cpl_diag::Span) -> Option<Arc<Facts>> {
        self.db.body_facts(path, at)
    }

    /// The file's diagnostics, from its memoised analysis.
    pub fn diagnostics(&mut self, path: &str) -> Vec<Diagnostic> {
        self.analysis(path)
            .map(|a| a.diagnostics.clone())
            .unwrap_or_default()
    }
}

impl Analysis {
    /// The session revision this analysis was built at.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// What checking found about each place in the file, in file positions.
    pub fn facts(&self) -> &Arc<Facts> {
        self.facts
            .get_or_init(|| Arc::new(Facts::from_analysis(self)))
    }
}

// Until every consumer asks the session by name, the database's queries
// (hover, definition, symbols, ...) are reached through it.
impl std::ops::Deref for Session {
    type Target = Database;
    fn deref(&self) -> &Database {
        &self.db
    }
}

impl std::ops::DerefMut for Session {
    fn deref_mut(&mut self) -> &mut Database {
        &mut self.db
    }
}

#[cfg(test)]
mod tests {
    use super::Session;
    use std::sync::Arc;

    #[test]
    fn answers_are_built_once_per_revision() {
        let mut s = Session::new();
        let r0 = s.revision();
        let r1 = s.set_file("a.cp", "let n = 3\nprint(n)\n");
        assert!(r1 > r0);
        let a = s.analysis("a.cp").unwrap();
        assert!(Arc::ptr_eq(&a, &s.analysis("a.cp").unwrap()));
        let f = s.facts("a.cp").unwrap();
        assert!(Arc::ptr_eq(&f, &s.facts("a.cp").unwrap()));
        assert_eq!(f.revision(), r1);
        let r2 = s.set_file("a.cp", "let n = 4\nprint(n)\n");
        assert!(r2 > r1);
        let g = s.facts("a.cp").unwrap();
        assert!(!Arc::ptr_eq(&f, &g));
        assert_eq!(g.revision(), r2);
        assert!(s.diagnostics("a.cp").is_empty());
        assert!(s.remove_file("a.cp") > r2);
        assert!(s.analysis("a.cp").is_none());
    }

    #[test]
    fn open_checks_one_file_once() {
        let a = super::open("b.cp", "let n: Int = true\n");
        assert_eq!(a.diagnostics.len(), 1);
        assert_eq!(a.facts().revision(), a.revision());
    }

    #[test]
    fn options_choose_the_loader() {
        struct Lib;
        impl crate::Loader for Lib {
            fn load(&self, _: &str, module: &[String]) -> Option<(String, String)> {
                (module == ["lib", "k"])
                    .then(|| ("lib/k.cp".into(), "pub fn k() -> Int = 7\n".into()))
            }
        }
        let mut s = Session::with_options(super::SessionOptions {
            libs: super::Libraries::Loader(Box::new(Lib)),
            edition: Some("2026".into()),
            ..Default::default()
        });
        assert_eq!(s.edition(), Some("2026"));
        s.set_file("m.cp", "use lib.k.{k}\nprint(k())\n");
        assert!(
            s.diagnostics("m.cp").is_empty(),
            "{:?}",
            s.diagnostics("m.cp")
        );
    }
}
