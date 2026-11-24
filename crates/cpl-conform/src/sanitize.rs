//! `cpl-conform sanitize [filter]`: no undefined behaviour in emitted std
//! code (handbook/domains/stdlib.md, "Correct results").
//!
//! Every std conformance case (`spec/conformance/std-*/`) and every std doc
//! example that compiles is emitted as C++, built with ASan, UBSan and
//! `_GLIBCXX_DEBUG`, and run on its input. A sanitizer or debug-mode report,
//! g++ rejecting the C++, or a run past its time limit is a finding; a trap
//! the program raises itself is not. Any finding fails the run.

use std::path::Path;

use cpl_check::Session;

use crate::conformance::{self, ConformanceCase};
use crate::diff::{compile_cpp_with, CppCompileError};
use crate::docs::{self, DocExample};
use crate::runner;

pub const FLAGS: &str =
    "-O1 -g -fsanitize=address,undefined -fno-sanitize-recover=all -D_GLIBCXX_DEBUG";

/// Seconds one sanitized run may take: sanitized C++ runs several times slower.
const LIMIT: u64 = 60;

enum Target {
    Case(ConformanceCase),
    Example(DocExample),
}

/// What one target came to.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Ran clean, to its end or to a trap of its own.
    Clean,
    /// Does not compile, so there is no C++ to run.
    NoCpp,
    Finding(String),
}

/// The report a run's stderr holds, if the sanitizers or debug mode stopped it.
pub fn report(stderr: &str) -> Option<String> {
    let at = ["runtime error:", "Sanitizer", "Error: attempt to"]
        .iter()
        .filter_map(|m| stderr.find(m))
        .min()?;
    let line_start = stderr[..at].rfind('\n').map_or(0, |i| i + 1);
    Some(runner::first_line_trimmed(&stderr[line_start..], 240))
}

fn run_cpp(cpp: &str, input: &[u8]) -> Outcome {
    let cache = conformance::cxx_cache("sanitize");
    let bin = match compile_cpp_with(cpp, &cache, FLAGS) {
        Ok(bin) => bin,
        Err(CppCompileError::Cxx(e)) => return Outcome::Finding(format!("g++ rejected it: {e}")),
        Err(CppCompileError::Infra(e)) => return Outcome::Finding(format!("g++ did not run: {e}")),
    };
    match runner::run_piped(std::process::Command::new(&bin), input, LIMIT) {
        Err(e) => Outcome::Finding(e),
        Ok(out) => match report(&String::from_utf8_lossy(&out.stderr)) {
            Some(r) => Outcome::Finding(r),
            None => Outcome::Clean,
        },
    }
}

fn run_one(db: &mut Session, root: &Path, target: &Target) -> (String, Outcome) {
    match target {
        Target::Case(case) => {
            let id = format!("conformance/{}", case.id);
            let Ok(text) = std::fs::read_to_string(&case.path) else {
                return (id, Outcome::Finding("cannot read the case".into()));
            };
            let path = case.path.to_string_lossy();
            let Ok(compiled) = runner::compile_program(db, &path, &text) else {
                return (id, Outcome::NoCpp);
            };
            let cpp = cpl_codegen::emit(&compiled.program, &path, Some(&text));
            (id, run_cpp(&cpp, &conformance::header_input(&text)))
        }
        Target::Example(ex) => {
            let id = format!("docs/{}", ex.id);
            let Some(code) = &ex.snippet else {
                return (id, Outcome::NoCpp);
            };
            let Ok((path, source, compiled)) = docs::compile_example(db, root, ex, code) else {
                return (id, Outcome::NoCpp);
            };
            let cpp = cpl_codegen::emit(&compiled.program, &path, Some(&source));
            (id, run_cpp(&cpp, ex.stdin.as_bytes()))
        }
    }
}

/// Runs every std case and doc example whose id contains `filter`; the outcome of each by id.
pub fn run(root: &Path, filter: Option<&str>) -> Vec<(String, Outcome)> {
    let cases = conformance::discover(root, filter.or(Some("std-")))
        .into_iter()
        .filter(|c| c.id.starts_with("std-"))
        .map(Target::Case);
    let examples = docs::discover(root, filter)
        .into_iter()
        .map(Target::Example);
    let targets: Vec<Target> = cases.chain(examples).collect();
    runner::parallel_with_db(&targets, |db, t| run_one(db, root, t))
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sanitizer_report_is_a_finding_and_a_trap_is_not() {
        let ub = "x.cpp:9:12: runtime error: signed integer overflow: 9223372036854775807 + 1\n";
        assert_eq!(report(ub).as_deref(), Some(ub.trim_end()));
        let asan =
            "=================\n==1==ERROR: AddressSanitizer: heap-buffer-overflow on address";
        assert!(report(asan)
            .unwrap()
            .starts_with("==1==ERROR: AddressSanitizer"));
        assert!(report("error[R0101]: index 5 is outside 0..3\n").is_none());
    }

    #[test]
    fn out_of_bounds_cpp_is_a_finding() {
        let cpp = "#include <vector>\nint main() { std::vector<int> v(3); return v[5]; }\n";
        assert!(matches!(run_cpp(cpp, b""), Outcome::Finding(_)));
        let ok = "#include <vector>\nint main() { std::vector<int> v(3); return v[2]; }\n";
        assert_eq!(run_cpp(ok, b""), Outcome::Clean);
    }
}
