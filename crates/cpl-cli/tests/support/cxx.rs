//! The one cache of g++ builds the test suites share: each build is kept in
//! `tests/` under `CPL_CXX_CACHE_DIR`, else `~/.cache/cpl/cxx`, keyed by a
//! hash of the compiler's `--version`, the flags and the C++, so suites that
//! build the same program with the same flags compile it once.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Once, OnceLock};
use std::time::SystemTime;

/// The most the kept builds may take: past it the least recently used go, so
/// the cache stays a cache and doesn't fill the disk (it grew 38 GB in a day).
const MAX_BYTES: u64 = 8 << 30;

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
    static EVICTED: Once = Once::new();
    EVICTED.call_once(|| evict(&cache, MAX_BYTES));
    if binary.exists() && std::fs::read(&source).is_ok_and(|kept| kept == cpp) {
        if let Ok(warnings) = std::fs::read_to_string(&said) {
            // A hit marks the build used, so eviction keeps it.
            let _ = std::fs::File::options()
                .append(true)
                .open(&binary)
                .and_then(|f| f.set_modified(SystemTime::now()));
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
        // Named by key, not by the temporary path, so the message is the same
        // on every machine and every run.
        let named = warnings.replace(&*tmp_source.to_string_lossy(), &format!("{key}.cpp"));
        return Err(format!("g++ failed on {key}.cpp:\n{named}"));
    }
    let tmp_said = cache.join(format!("{unique}.stderr"));
    std::fs::write(&tmp_said, &warnings).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_binary, &binary).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_source, &source).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_said, &said).map_err(|e| e.to_string())?;
    Ok((binary, warnings))
}

/// Drop the least recently used builds in `dir` (a build's binary, C++ and
/// stderr go together) until what is kept fits in `max_bytes`. Run once per
/// test process, so a run pays one listing, not one per build.
fn evict(dir: &Path, max_bytes: u64) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut builds: std::collections::HashMap<String, (u64, SystemTime)> = Default::default();
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let name = entry.file_name().to_string_lossy().into_owned();
        let key = name.split('.').next().unwrap_or_default().to_owned();
        let build = builds.entry(key).or_insert((0, SystemTime::UNIX_EPOCH));
        build.0 += meta.len();
        if !name.contains('.') {
            build.1 = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        }
    }
    let mut total: u64 = builds.values().map(|b| b.0).sum();
    let mut by_use: Vec<_> = builds.into_iter().collect();
    by_use.sort_by_key(|(_, (_, used))| *used);
    for (key, (bytes, _)) in by_use {
        if total <= max_bytes {
            break;
        }
        for name in [key.clone(), format!("{key}.cpp"), format!("{key}.stderr")] {
            let _ = std::fs::remove_file(dir.join(name));
        }
        total -= bytes;
    }
}

#[test]
fn eviction_drops_the_least_recently_used_builds_first() {
    let dir = std::env::temp_dir().join(format!("cpl-cxx-evict-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (i, key) in ["old", "mid", "new"].into_iter().enumerate() {
        for name in [
            key.to_owned(),
            format!("{key}.cpp"),
            format!("{key}.stderr"),
        ] {
            std::fs::write(dir.join(&name), [0u8; 100]).unwrap();
        }
        let used = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000 * (i as u64 + 1));
        std::fs::File::options()
            .append(true)
            .open(dir.join(key))
            .unwrap()
            .set_modified(used)
            .unwrap();
    }
    evict(&dir, 650);
    let mut left: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    left.sort();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(
        left,
        [
            "mid",
            "mid.cpp",
            "mid.stderr",
            "new",
            "new.cpp",
            "new.stderr"
        ]
    );
}
