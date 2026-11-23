//! What the sanitized differential tests share: a seeded generator, a
//! runner that feeds stdin, and a cache of sanitized binaries keyed by a
//! hash of the C++, the flags and the compiler.

use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

pub const GXX_FLAGS: [&str; 5] = [
    "-std=c++17",
    "-fsanitize=address,undefined",
    "-fno-sanitize-recover=all",
    "-w",
    "-o",
];

/// xorshift64*: the same cases on every machine.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1)
    }
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + self.below((hi - lo + 1) as u64) as i64
    }
    pub fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.below(xs.len() as u64) as usize]
    }
}

pub fn run(mut command: Command, input: &str) -> std::process::Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("starts");
    // A program that stops early closes its input; that is not an error.
    let _ = child
        .stdin
        .take()
        .expect("piped")
        .write_all(input.as_bytes());
    child.wait_with_output().expect("exits")
}

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

/// The sanitized binary of `cpp`, kept in `cache`, compiled now or kept from an earlier run.
pub fn sanitized_binary(cache: &Path, cpp: &[u8]) -> Result<PathBuf, String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (gxx_version(), GXX_FLAGS, cpp).hash(&mut hasher);
    let key = format!("{:016x}", hasher.finish());
    std::fs::create_dir_all(cache).map_err(|e| e.to_string())?;
    let (source, binary) = (cache.join(format!("{key}.cpp")), cache.join(&key));
    if binary.exists() && std::fs::read(&source).is_ok_and(|kept| kept == cpp) {
        return Ok(binary);
    }
    let unique = format!(
        "{key}.{}.{:?}",
        std::process::id(),
        std::thread::current().id()
    )
    .replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "");
    let (tmp_source, tmp_binary) = (cache.join(format!("{unique}.cpp")), cache.join(&unique));
    std::fs::write(&tmp_source, cpp).map_err(|e| e.to_string())?;
    let compiled = Command::new("g++")
        .args(GXX_FLAGS)
        .arg(&tmp_binary)
        .arg(&tmp_source)
        .output()
        .map_err(|e| format!("g++ did not start ({e}); these tests need it"))?;
    if !compiled.status.success() {
        return Err(format!(
            "g++ failed on {}:\n{}",
            tmp_source.display(),
            String::from_utf8_lossy(&compiled.stderr)
        ));
    }
    std::fs::rename(&tmp_binary, &binary).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_source, &source).map_err(|e| e.to_string())?;
    Ok(binary)
}
