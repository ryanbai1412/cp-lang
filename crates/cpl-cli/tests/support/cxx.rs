//! The one cache of g++ builds the test suites share: each build is kept in
//! `tests/` under `CPL_CXX_CACHE_DIR`, else `~/.cache/cpl/cxx`, keyed by a
//! hash of the compiler's `--version`, the flags and the C++, so suites that
//! build the same program with the same flags compile it once.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// A cached build: the binary, the C++ beside it, and what g++ printed
/// while building it (kept, so a warning still shows on a later hit).
fn cache_dir() -> PathBuf {
    let root = match cpl_env::CXX_CACHE_DIR.path() {
        Some(dir) => dir,
        None => match std::env::var_os("HOME") {
            Some(home) => Path::new(&home).join(".cache/cpl/cxx"),
            None => std::env::temp_dir().join("cpl-cxx"),
        },
    };
    root.join("tests")
}

/// What `g++ --version` prints, so a compiler upgrade rebuilds every binary.
fn gxx_version() -> &'static [u8] {
    static VERSION: OnceLock<Vec<u8>> = OnceLock::new();
    VERSION.get_or_init(|| {
        Command::new("g++")
            .arg("--version")
            .output()
            .map(|out| out.stdout)
            .unwrap_or_default()
    })
}

/// FNV-1a, stable across Rust releases, so the cache outlives a toolchain.
fn key(flags: &[&str], cpp: &[u8]) -> String {
    let flags = flags.join("\0");
    let parts = [gxx_version(), flags.as_bytes(), cpp];
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for byte in part.iter().chain(b"\0\x01") {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

/// `cpp` built by `g++ <flags> -o <binary>`, compiled now or kept from an
/// earlier build of the same bytes with the same flags and compiler.
/// The binary g++ builds from `cpp` with `flags`, and what g++ printed while building it.
pub fn build(cpp: &[u8], flags: &[&str]) -> Result<(PathBuf, String), String> {
    let cache = cache_dir();
    std::fs::create_dir_all(&cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let key = key(flags, cpp);
    let (source, binary, said) = (
        cache.join(format!("{key}.cpp")),
        cache.join(&key),
        cache.join(format!("{key}.stderr")),
    );
    if binary.exists() && std::fs::read(&source).is_ok_and(|kept| kept == cpp) {
        if let Ok(warnings) = std::fs::read_to_string(&said) {
            return Ok((binary, warnings));
        }
    }
    // Built under a name of its own and renamed, so no test runs a binary
    // another is still writing.
    let unique = format!(
        "{key}.{}.{:?}",
        std::process::id(),
        std::thread::current().id()
    )
    .replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "");
    let (tmp_source, tmp_binary) = (cache.join(format!("{unique}.cpp")), cache.join(&unique));
    std::fs::write(&tmp_source, cpp).map_err(|e| e.to_string())?;
    let compiled = Command::new("g++")
        .args(flags)
        .arg("-o")
        .arg(&tmp_binary)
        .arg(&tmp_source)
        .output()
        .map_err(|e| format!("g++ did not start ({e}); these tests need it"))?;
    let warnings = String::from_utf8_lossy(&compiled.stderr).into_owned();
    if !compiled.status.success() {
        let _ = std::fs::remove_file(&tmp_binary);
        return Err(format!(
            "g++ failed on {}:\n{warnings}",
            tmp_source.display()
        ));
    }
    let tmp_said = cache.join(format!("{unique}.stderr"));
    std::fs::write(&tmp_said, &warnings).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_binary, &binary).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_source, &source).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_said, &said).map_err(|e| e.to_string())?;
    Ok((binary, warnings))
}
