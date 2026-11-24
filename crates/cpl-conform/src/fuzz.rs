//! `cpl-conform fuzz`: generated programs on both engines at every
//! `--checks` level. A program from [`crate::gen`] is valid by
//! construction, so it must check, run on the interpreter without a trap,
//! and print the same bytes from the C++ built at `none`, `traps` and
//! `all`. Enforces compiler-changes/R4 in
//! `handbook/guidelines/compiler-changes-checklist.md`.

use std::path::Path;

use cpl_check::Session;
use cpl_codegen::{Checks, Target};

use crate::diff::{compile_cpp, run_binary, CppCompileError};
use crate::gen;
use crate::runner::{self, CompileError};

/// What went wrong with one generated program.
#[derive(Debug, Clone)]
pub struct Finding {
    pub seed: u64,
    /// `rejected`, `interp_trap`, `cxx_err`, `cpp_trap`, `mismatch` or `infra`.
    pub kind: &'static str,
    pub detail: String,
}

/// Runs the program of `seed`: on the interpreter, and as C++ at each of
/// `levels` when `cxx` is set.
pub fn check(
    db: &mut Session,
    cache_dir: &Path,
    seed: u64,
    levels: &[Checks],
    cxx: bool,
) -> Result<(), Finding> {
    let text = gen::program(seed);
    let path = format!("gen_{seed}.cp");
    let found = |kind, detail: String| Finding { seed, kind, detail };
    let compiled = match runner::compile_program(db, &path, &text) {
        Ok(c) => c,
        Err(CompileError::Diagnostics(d)) => return Err(found("rejected", d)),
    };
    let run = runner::run_interp_bounded(&compiled.program, &path, &text, b"")
        .map_err(|e| found("interp_trap", e))?;
    if run.trap.is_some() {
        return Err(found("interp_trap", run.trap_rendered));
    }
    if !cxx {
        return Ok(());
    }
    let want = String::from_utf8_lossy(&run.stdout).into_owned();
    for &checks in levels {
        let target = Target {
            checks,
            ..Target::default()
        };
        let cpp = cpl_codegen::emit_for(&compiled.program, &path, Some(&text), &target);
        let level = Checks::NAMES[checks as usize];
        let bin = match compile_cpp(&cpp, cache_dir) {
            Ok(b) => b,
            Err(CppCompileError::Cxx(e)) => return Err(found("cxx_err", format!("{level}: {e}"))),
            Err(CppCompileError::Infra(e)) => return Err(found("infra", e)),
        };
        let got = run_binary(&bin, b"").map_err(|e| found("cpp_trap", format!("{level}: {e}")))?;
        let got = String::from_utf8_lossy(&got);
        if got.trim_end() != want.trim_end() {
            return Err(found(
                "mismatch",
                format!(
                    "{level}: interp={:?} cpp={:?}",
                    runner::truncate_chars(want.trim(), 120),
                    runner::truncate_chars(got.trim(), 120)
                ),
            ));
        }
    }
    Ok(())
}

/// Checks seeds `start..start + count` on all threads. Each finding's
/// program is written to `keep` as `<seed>.cp`.
pub fn run(start: u64, count: u64, cxx: bool, keep: &Path) -> Vec<Finding> {
    let levels = [Checks::None, Checks::Traps, Checks::All];
    let cache_dir = std::env::temp_dir().join("cpl-conform-fuzz-cxx");
    let seeds: Vec<u64> = (start..start + count).collect();
    let results = runner::parallel_with_db(&seeds, |db, &seed| {
        let found = check(db, &cache_dir, seed, &levels, cxx).err();
        if found.is_some() {
            let _ = std::fs::create_dir_all(keep);
            let _ = std::fs::write(keep.join(format!("{seed}.cp")), gen::program(seed));
        }
        (format!("gen_{seed}"), found)
    });
    let mut out: Vec<Finding> = results.into_values().flatten().collect();
    out.sort_by_key(|f| f.seed);
    out
}
