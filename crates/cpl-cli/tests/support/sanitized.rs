//! What the sanitized differential tests share: a seeded generator, a
//! runner that feeds stdin, and sanitized builds from the shared C++ cache.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[path = "cxx.rs"]
mod cxx;

pub const GXX_FLAGS: [&str; 4] = [
    "-std=c++17",
    "-fsanitize=address,undefined",
    "-fno-sanitize-recover=all",
    "-w",
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

/// The sanitized binary of `cpp`, from the shared C++ cache or compiled now.
pub fn sanitized_binary(cpp: &[u8]) -> Result<PathBuf, String> {
    cxx::build(cpp, &GXX_FLAGS).map(|(binary, _)| binary)
}
