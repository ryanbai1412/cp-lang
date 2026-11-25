//! The one place a `CPL_*` environment variable is read. Each variable is a
//! [`Var`] declared here with what it does, so `ALL` lists every one and no
//! other crate names one to `std::env` (debt-ratchet's `cpl_env_reads`).
//! The user's home folder is found here too ([`home_dir`]).

use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;

/// `HOME`, or `USERPROFILE` on Windows, where `HOME` is usually unset.
/// The one home lookup: libraries, setup, toolchains, crash reports, caches.
pub fn home_dir() -> Option<PathBuf> {
    (["HOME", "USERPROFILE"].into_iter())
        .filter_map(std::env::var_os)
        .find(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// One `CPL_*` variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Var {
    name: &'static str,
    doc: &'static str,
}

impl Var {
    /// The variable's name, for a child process's environment.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// What the variable does.
    pub const fn doc(&self) -> &'static str {
        self.doc
    }

    /// The value as set, empty included.
    pub fn raw(&self) -> Option<OsString> {
        std::env::var_os(self.name)
    }

    /// Whether it is set at all, even to nothing.
    pub fn is_set(&self) -> bool {
        self.raw().is_some()
    }

    /// Whether it is set to exactly `value`.
    pub fn is(&self, value: &str) -> bool {
        self.raw().is_some_and(|v| v == value)
    }

    /// The value as text, unless unset, blank or not UTF-8.
    pub fn text(&self) -> Option<String> {
        let text = self.raw()?.into_string().ok()?;
        (!text.trim().is_empty()).then_some(text)
    }

    /// The value as a path, unless unset or empty.
    pub fn path(&self) -> Option<PathBuf> {
        self.raw().filter(|v| !v.is_empty()).map(PathBuf::from)
    }

    /// The value parsed as a `T`, unless unset, blank or malformed.
    pub fn parse<T: FromStr>(&self) -> Option<T> {
        self.text()?.trim().parse().ok()
    }
}

macro_rules! vars {
    ($($(#[doc = $doc:literal])+ $id:ident = $name:literal;)+) => {
        $($(#[doc = $doc])+ pub const $id: Var = Var { name: $name, doc: concat!($($doc),+) };)+
        /// Every variable, in the order declared.
        pub const ALL: &[Var] = &[$($id),+];
    };
}

vars! {
    /// The `std/` directory to load in place of the built-in one; set empty for none.
    STD_DIR = "CPL_STD_DIR";
    /// At build time: the commit stamped into `cpl --version` (default: git's HEAD).
    COMMIT = "CPL_COMMIT";
    /// At build time: the release channel stamped into `cpl --version` (default `dev`).
    CHANNEL = "CPL_CHANNEL";
    /// Where crash reports go (default `~/.cpl/crashes`).
    CRASH_DIR = "CPL_CRASH_DIR";
    /// The VS Code command `cpl setup` installs the extension with (default `code`).
    CODE = "CPL_CODE";
    /// The shell command `--copy` pipes the C++ into, in place of the system clipboard tool.
    CLIPBOARD = "CPL_CLIPBOARD";
    /// The toolchain home (default `~/.cpl`).
    HOME = "CPL_HOME";
    /// Where toolchain releases are downloaded from.
    RELEASES_URL = "CPL_RELEASES_URL";
    /// The toolchain version to run, over the project's pin.
    VERSION = "CPL_VERSION";
    /// Set by the launcher: the toolchain that is running.
    TOOLCHAIN = "CPL_TOOLCHAIN";
    /// Tests rewrite their goldens from what they see.
    BLESS = "CPL_BLESS";
    /// The VM fallback test seeds its empty list from what it sees.
    BLESS_VM_FALLBACKS = "CPL_BLESS_VM_FALLBACKS";
    /// How many rounds the front-end fuzz test runs.
    FUZZ_ROUNDS = "CPL_FUZZ_ROUNDS";
    /// How many mutants of each program the lint fuzz test checks.
    FUZZ_MUTANTS = "CPL_FUZZ_MUTANTS";
    /// The directory the lint fuzz test writes each mutant to.
    FUZZ_DUMP = "CPL_FUZZ_DUMP";
    /// How many edits per port the LSP fuzz test makes.
    LSP_FUZZ_EDITS = "CPL_LSP_FUZZ_EDITS";
    /// How many ports the LSP fuzz test edits.
    LSP_FUZZ_PORTS = "CPL_LSP_FUZZ_PORTS";
    /// The Chrome the browser trace test drives (default: one on `PATH`).
    CHROME = "CPL_CHROME";
    /// The engines (comma-separated, or `all`) the highlighting agreement test must run, not skip.
    HIGHLIGHT = "CPL_HIGHLIGHT";
    /// `1`: conformance runs the release `cpl` binary.
    CONFORM_RELEASE = "CPL_CONFORM_RELEASE";
    /// The `cpl` binary conformance runs.
    CONFORM_CLI = "CPL_CONFORM_CLI";
    /// The flags conformance compiles the C++ with (default `-O2`).
    CONFORM_CXXFLAGS = "CPL_CONFORM_CXXFLAGS";
    /// Seconds the conformance run may take in all (default 900).
    CONFORM_BUDGET = "CPL_CONFORM_BUDGET";
    /// Seconds one conformance case may take (default 20).
    CONFORM_CASE_TIMEOUT = "CPL_CONFORM_CASE_TIMEOUT";
    /// The memory, in MB, a conformance case may use.
    CONFORM_MAX_RSS_MB = "CPL_CONFORM_MAX_RSS_MB";
    /// Where compiled C++ is cached across runs (default: a temporary directory).
    CXX_CACHE_DIR = "CPL_CXX_CACHE_DIR";
    /// A fingerprint of the compiler and system headers, for a cache that outlives them.
    CXX_SYSTEM_KEY = "CPL_CXX_SYSTEM_KEY";
    /// Where judge probes are remembered; set empty for nowhere.
    JUDGE_PROBE_DIR = "CPL_JUDGE_PROBE_DIR";
    /// Where stress caches compiled programs (default: a temporary directory).
    CACHE = "CPL_CACHE";
    /// Set: stress compiles without a precompiled header.
    NO_PCH = "CPL_NO_PCH";
    /// The VM prints its op counts on exit; `json` prints them as JSON.
    VM_STATS = "CPL_VM_STATS";
    /// `0`: making a package's cases never fetches a g++ 14 with
    /// `scripts/gxx14.sh` when none is on `PATH`.
    GXX_FETCH = "CPL_GXX_FETCH";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_var_is_named_once_and_says_what_it_does() {
        let mut names: Vec<&str> = ALL.iter().map(Var::name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), ALL.len(), "a name declared twice");
        for var in ALL {
            assert!(var.name().starts_with("CPL_"), "{}", var.name());
            assert!(!var.doc().trim().is_empty(), "{} has no doc", var.name());
        }
    }

    #[test]
    fn a_value_reads_as_text_path_number_or_flag() {
        let var = Var {
            name: "CPL_ENV_TEST_VALUE",
            doc: "",
        };
        assert_eq!((var.raw(), var.is_set(), var.text()), (None, false, None));
        std::env::set_var(var.name(), " ");
        assert!(var.is_set() && var.is(" "));
        assert_eq!((var.text(), var.parse::<u32>()), (None, None));
        assert_eq!(var.path(), Some(PathBuf::from(" ")));
        std::env::set_var(var.name(), "");
        assert_eq!((var.path(), var.raw()), (None, Some(OsString::new())));
        std::env::set_var(var.name(), " 42 ");
        assert_eq!(var.parse::<u32>(), Some(42));
        assert_eq!(var.text().as_deref(), Some(" 42 "));
        assert!(!var.is("42"));
        std::env::set_var(var.name(), "x");
        assert_eq!(var.parse::<u32>(), None);
        std::env::remove_var(var.name());
    }
}
