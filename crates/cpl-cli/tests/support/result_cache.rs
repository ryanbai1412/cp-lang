//! A per-program result cache for tests that build every shipped program:
//! a program's result is stored under a hash of its source and of what
//! decides the result (the test binary, which links the compiler, `std/`
//! and the `CPL_*` variables the compiler reads), so a warm run rebuilds only
//! the programs that changed; the newest `KEEP` sets of inputs keep theirs. It lives in `~/.cache/cpl/<name>` beside the
//! C++ cache (`$XDG_CACHE_HOME/cpl/<name>` when set); with no home it is off.

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The variables that change what `cpl build` and `cpl explain` print.
const ENV: &[&str] = &["CPL_STD_DIR", "CPL_STD_SOURCE", "CPL_CHANNEL"];

pub struct ResultCache {
    dir: Option<PathBuf>,
    inputs: u128,
    misses: AtomicUsize,
}

/// A 128-bit hash of `parts`, from two keyed SipHashes.
fn hash128(parts: &[&[u8]]) -> u128 {
    let half = |salt: u8| {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        salt.hash(&mut h);
        for p in parts {
            p.hash(&mut h);
        }
        h.finish()
    };
    (u128::from(half(1)) << 64) | u128::from(half(2))
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else {
            out.push(p);
        }
    }
}

/// The running binary's content hash, remembered on disk by its inode, size and
/// mtime so each test process need not read it again.
fn exe_hash(dir: Option<&Path>) -> u128 {
    let exe = std::env::current_exe().expect("current_exe");
    #[cfg(unix)]
    let memo = {
        use std::os::unix::fs::MetadataExt;
        let m = std::fs::metadata(&exe).expect("metadata of the test binary");
        dir.map(|d| {
            d.join(format!(
                "exe-{}-{}-{}-{}.{}",
                m.dev(),
                m.ino(),
                m.size(),
                m.mtime(),
                m.mtime_nsec()
            ))
        })
    };
    #[cfg(not(unix))]
    let memo: Option<PathBuf> = None;
    let _ = dir;
    if let Some(h) = memo
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| u128::from_str_radix(s.trim(), 16).ok())
    {
        return h;
    }
    let h = hash128(&[&std::fs::read(&exe).expect("read the test binary")]);
    if let Some(p) = memo {
        write_atomic(&p, format!("{h:032x}").as_bytes());
    }
    h
}

/// How many sets of inputs (builds of the test binary, versions of std) keep their results.
const KEEP: usize = 8;

/// Deletes all but the `keep` newest entries of `dir` other than `except`.
fn prune(dir: &Path, keep: usize, except: &str) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut found: Vec<_> = entries
        .flatten()
        .filter(|e| e.file_name() != except)
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    found.sort_by_key(|a| std::cmp::Reverse(a.0));
    for (_, p) in found.into_iter().skip(keep) {
        let _ = if p.is_dir() {
            std::fs::remove_dir_all(&p)
        } else {
            std::fs::remove_file(&p)
        };
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) {
    let tmp = path.with_extension(format!(
        "tmp{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    if std::fs::write(&tmp, bytes).is_ok() && std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// `~/.cache/cpl/<name>`, or none when off.
pub fn default_dir(name: &str) -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|v| !v.is_empty())
                .map(|h| PathBuf::from(h).join(".cache"))
        })?;
    Some(base.join("cpl").join(name))
}

impl ResultCache {
    /// A cache in `dir` (none: every lookup computes) for results that depend on
    /// the test binary, `repo`'s `std/` and the compiler's variables.
    pub fn new(dir: Option<PathBuf>, repo: &Path) -> Self {
        let dir = dir.filter(|d| std::fs::create_dir_all(d).is_ok());
        let mut std_files = Vec::new();
        files(&repo.join("std"), &mut std_files);
        std_files.sort();
        let memos = dir
            .as_ref()
            .map(|d| d.join("exe"))
            .filter(|d| std::fs::create_dir_all(d).is_ok());
        let mut parts: Vec<Vec<u8>> = vec![exe_hash(memos.as_deref()).to_le_bytes().to_vec()];
        for f in &std_files {
            parts.push(
                f.strip_prefix(repo)
                    .unwrap_or(f)
                    .to_string_lossy()
                    .into_owned()
                    .into_bytes(),
            );
            parts.push(std::fs::read(f).unwrap_or_default());
        }
        for v in ENV {
            parts.push(format!("{v}={:?}", std::env::var_os(v)).into_bytes());
        }
        let parts: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();
        let inputs = hash128(&parts);
        let dir = dir.map(|d| {
            prune(&d, KEEP, "exe");
            if let Some(m) = &memos {
                prune(m, KEEP, "");
            }
            d.join(format!("{inputs:032x}"))
        });
        let dir = dir.filter(|d| std::fs::create_dir_all(d).is_ok());
        Self {
            dir,
            inputs,
            misses: AtomicUsize::new(0),
        }
    }

    /// How many lookups computed their result.
    pub fn misses(&self) -> usize {
        self.misses.load(Ordering::Relaxed)
    }

    /// `src`'s stored lines, or `compute()`'s, stored for next time.
    pub fn lines(&self, src: &Path, compute: impl FnOnce() -> Vec<String>) -> Vec<String> {
        let text = std::fs::read(src).unwrap_or_default();
        let key = hash128(&[
            &self.inputs.to_le_bytes(),
            src.to_string_lossy().as_bytes(),
            &text,
        ]);
        let entry = self.dir.as_ref().map(|d| d.join(format!("{key:032x}")));
        if let Some(stored) = entry.as_ref().and_then(|e| std::fs::read_to_string(e).ok()) {
            return stored.lines().map(str::to_owned).collect();
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        let lines = compute();
        if let Some(e) = entry.filter(|_| lines.iter().all(|l| !l.contains(['\n', '\r']))) {
            write_atomic(
                &e,
                lines
                    .iter()
                    .map(|l| format!("{l}\n"))
                    .collect::<String>()
                    .as_bytes(),
            );
        }
        lines
    }
}
