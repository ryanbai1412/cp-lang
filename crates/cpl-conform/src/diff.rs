use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Condvar, Mutex};
use std::time::Duration;

use cpl_check::Database;

use crate::conformance::{self, Expect};
use crate::corpus;
use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};
use crate::variants;

#[derive(Clone, Debug)]
enum DiffTarget {
    Conformance {
        id: String,
        path: PathBuf,
    },
    Corpus {
        id: String,
        spec_path: PathBuf,
        tests: Vec<PathBuf>,
    },
    /// A corpus `port.cp` with side-effect probes (`variants`), on its smallest input.
    Variant {
        id: String,
        port_path: PathBuf,
        input: PathBuf,
    },
}

fn discover(root: &Path, filter: Option<&str>) -> Vec<DiffTarget> {
    let mut targets = Vec::new();

    for case in conformance::discover(root, None) {
        let id = format!("conformance/{}", case.id);
        if let Some(f) = filter {
            if !id.contains(f) {
                continue;
            }
        }
        let Ok(text) = std::fs::read_to_string(&case.path) else {
            continue;
        };
        if matches!(
            conformance::parse_header(&text),
            Ok((Expect::OkOutput(_), _))
        ) {
            targets.push(DiffTarget::Conformance {
                id,
                path: case.path,
            });
        }
    }

    for prob in corpus::discover(root, None) {
        if prob.tests.is_empty() {
            continue;
        }
        let id = format!("corpus/{}", prob.id);
        if let Some(f) = filter {
            if !id.contains(f) {
                continue;
            }
        }
        targets.push(DiffTarget::Corpus {
            id,
            spec_path: prob.spec_path,
            tests: prob.tests.into_iter().map(|(inp, _)| inp).collect(),
        });
    }

    for prob in corpus::discover_file(root, None, "port.cp") {
        let id = format!("variant/{}", prob.id);
        if filter.is_some_and(|f| !id.contains(f)) {
            continue;
        }
        let smallest = prob
            .tests
            .into_iter()
            .map(|(inp, _)| inp)
            .min_by_key(|inp| {
                (
                    std::fs::metadata(inp).map(|m| m.len()).unwrap_or(u64::MAX),
                    inp.clone(),
                )
            });
        if let Some(input) = smallest {
            targets.push(DiffTarget::Variant {
                id,
                port_path: prob.spec_path,
                input,
            });
        }
    }

    targets
        .into_iter()
        .enumerate()
        .filter(|(i, _)| runner::in_shard(*i))
        .map(|(_, t)| t)
        .collect()
}

fn fnv1a_hex(data: &[u8]) -> String {
    let mut h1: u64 = 0xcbf29ce484222325;
    let mut h2: u64 = 0x84222325cbf29ce4;
    for &b in data {
        h1 ^= u64::from(b);
        h1 = h1.wrapping_mul(0x100000001b3);
        h2 = h2.wrapping_mul(0x100000001b3);
        h2 ^= u64::from(b);
    }
    format!("{h1:016x}{h2:016x}")
}

enum CppCompileError {
    Cxx(String),
    Infra(String),
}

struct GxxGate {
    active: Mutex<usize>,
    cv: Condvar,
    max: usize,
}

impl GxxGate {
    const fn new(max: usize) -> Self {
        Self {
            active: Mutex::new(0),
            cv: Condvar::new(),
            max,
        }
    }

    fn acquire(&self) -> GxxPermit<'_> {
        let mut active = self.active.lock().unwrap();
        while *active >= self.max {
            active = self.cv.wait(active).unwrap();
        }
        *active += 1;
        GxxPermit(self)
    }
}

struct GxxPermit<'a>(&'a GxxGate);

impl Drop for GxxPermit<'_> {
    fn drop(&mut self) {
        let mut active = self.0.active.lock().unwrap();
        *active = active.saturating_sub(1);
        self.0.cv.notify_one();
    }
}

static GXX_GATE: GxxGate = GxxGate::new(6);

fn is_transient_gxx_failure(code: Option<i32>, stderr: &str) -> bool {
    if code.is_none() {
        return true;
    }
    let lower = stderr.to_ascii_lowercase();
    lower.contains("cannot allocate memory")
        || lower.contains("resource temporarily unavailable")
        || lower.contains("killed")
        || lower.contains("fork:")
        || lower.contains("virtual memory exhausted")
}

fn compile_cpp(cpp: &str, cache_dir: &Path) -> Result<PathBuf, CppCompileError> {
    std::fs::create_dir_all(cache_dir)
        .map_err(|e| CppCompileError::Infra(format!("cannot create cache dir: {e}")))?;
    // `CPL_CONFORM_CXXFLAGS` replaces `-O2`, e.g. `-O1 -g -fsanitize=address,undefined`
    // to turn undefined behaviour in emitted C++ into a failing case.
    let flags = std::env::var("CPL_CONFORM_CXXFLAGS").unwrap_or_else(|_| "-O2".into());
    // `CPL_CXX_SYSTEM_KEY` fingerprints the compiler and system headers when the
    // cache outlives them (shared across checkouts and CI runs).
    let system = std::env::var("CPL_CXX_SYSTEM_KEY").unwrap_or_default();
    let key = if flags == "-O2" && system.is_empty() {
        fnv1a_hex(cpp.as_bytes())
    } else {
        fnv1a_hex(format!("{system}\0{flags}\0{cpp}").as_bytes())
    };
    let bin = cache_dir.join(&key);
    if bin.is_file() {
        return Ok(bin);
    }
    let tid = std::thread::current().id();
    let unique = format!("{key}_{}_{tid:?}", std::process::id())
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let src = cache_dir.join(format!("{unique}.cpp"));
    let tmp_bin = cache_dir.join(format!("{unique}.bin"));
    std::fs::write(&src, cpp)
        .map_err(|e| CppCompileError::Infra(format!("cannot write cpp: {e}")))?;

    let mut last_infra: Option<String> = None;
    for attempt in 0..3 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_millis(150 * attempt as u64));
            if bin.is_file() {
                let _ = std::fs::remove_file(&src);
                return Ok(bin);
            }
        }
        let _permit = GXX_GATE.acquire();
        let out = match Command::new("g++")
            .args(flags.split_whitespace())
            .args(["-std=c++17", "-o"])
            .arg(&tmp_bin)
            .arg(&src)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                last_infra = Some(format!("cannot run g++: {e}"));
                continue;
            }
        };
        if out.status.success() {
            let _ = std::fs::remove_file(&src);
            let _ = std::fs::rename(&tmp_bin, &bin);
            return if bin.is_file() { Ok(bin) } else { Ok(tmp_bin) };
        }
        let _ = std::fs::remove_file(&tmp_bin);
        let stderr = String::from_utf8_lossy(&out.stderr);
        if is_transient_gxx_failure(out.status.code(), &stderr) {
            last_infra = Some(format!(
                "g++ transient failure ({:?}): {}",
                out.status.code(),
                runner::first_line_trimmed(&stderr, 160)
            ));
            continue;
        }
        let _ = std::fs::remove_file(&src);
        return Err(CppCompileError::Cxx(runner::first_line_trimmed(
            &stderr, 160,
        )));
    }
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&tmp_bin);
    Err(CppCompileError::Infra(
        last_infra.unwrap_or_else(|| "g++ failed after retries".into()),
    ))
}

fn run_binary(bin: &Path, input: &[u8]) -> Result<Vec<u8>, String> {
    let out = runner::run_piped(Command::new(bin), input, runner::case_timeout())
        .map_err(|e| format!("{}: {e}", bin.display()))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "exit {:?}: {}",
            out.status.code(),
            runner::first_line_trimmed(&stderr, 120)
        ));
    }
    Ok(out.stdout)
}

fn run_one(db: &mut Database, cache_dir: &Path, target: &DiffTarget) -> (String, CaseOutcome) {
    let (id, path, inputs, token_compare) = match target {
        DiffTarget::Conformance { id, path } => (
            id.clone(),
            path.clone(),
            vec![(
                String::new(),
                std::fs::read_to_string(path)
                    .map(|t| conformance::header_input(&t))
                    .unwrap_or_default(),
            )],
            false,
        ),
        DiffTarget::Corpus {
            id,
            spec_path,
            tests,
        } => {
            let mut inps = Vec::with_capacity(tests.len());
            for t in tests {
                let name = t
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                match std::fs::read(t) {
                    Ok(bytes) => inps.push((name, bytes)),
                    Err(e) => {
                        return (
                            id.clone(),
                            CaseOutcome {
                                status: "error".into(),
                                detail: format!("cannot read {name}: {e}"),
                            },
                        );
                    }
                }
            }
            (id.clone(), spec_path.clone(), inps, true)
        }
        DiffTarget::Variant {
            id,
            port_path,
            input,
        } => match std::fs::read(input) {
            Ok(bytes) => (
                id.clone(),
                port_path.clone(),
                vec![(
                    input
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    bytes,
                )],
                true,
            ),
            Err(e) => {
                return (
                    id.clone(),
                    CaseOutcome {
                        status: "error".into(),
                        detail: format!("cannot read the input: {e}"),
                    },
                );
            }
        },
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            return (
                id,
                CaseOutcome {
                    status: "error".into(),
                    detail: format!("cannot read: {e}"),
                },
            );
        }
    };
    let path_str = path.to_string_lossy();
    let text = if matches!(target, DiffTarget::Variant { .. }) {
        let skip = |detail: &str| CaseOutcome {
            status: "skip".into(),
            detail: detail.into(),
        };
        if runner::compile_program(db, &path_str, &text).is_err() {
            return (id, skip("the port does not compile yet"));
        }
        let Some((variant, _)) = variants::build(db, &path_str, &text) else {
            return (id, skip("no probe checks"));
        };
        // Kept for reproducing a mismatch: `cpl run` it next to the port.
        let dir = cache_dir.join("variants");
        let _ = std::fs::create_dir_all(&dir).and_then(|_| {
            std::fs::write(dir.join(format!("{}.cp", id.replace('/', "_"))), &variant)
        });
        variant
    } else {
        text
    };
    let compiled = match runner::compile_program(db, &path_str, &text) {
        Ok(c) => c,
        Err(CompileError::Diagnostics(_)) => {
            // Guard against cross-case Database state contamination under parallel reuse:
            // retry once on a fresh Database before classifying as skip.
            let mut fresh = Database::new();
            match runner::compile_program(&mut fresh, &path_str, &text) {
                Ok(c) => c,
                Err(CompileError::Diagnostics(diag)) => {
                    return (
                        id,
                        CaseOutcome {
                            status: "skip".into(),
                            detail: format!(
                                "does not compile yet: {}",
                                runner::first_line_trimmed(&diag, 140)
                            ),
                        },
                    );
                }
            }
        }
    };

    let mut interp_outputs = Vec::with_capacity(inputs.len());
    for (_, inp) in &inputs {
        let run = match runner::run_interp_bounded(&compiled.program, &path_str, &text, inp) {
            Ok(run) => run,
            Err(detail) => {
                return (
                    id,
                    CaseOutcome {
                        status: "error".into(),
                        detail,
                    },
                )
            }
        };
        if run.trap.is_some() {
            return (
                id,
                CaseOutcome {
                    status: "skip".into(),
                    detail: "interpreter trapped".into(),
                },
            );
        }
        interp_outputs.push(run.stdout);
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("main.cp");
    let cpp = cpl_codegen::emit(&compiled.program, name, Some(&text));
    let bin = match compile_cpp(&cpp, cache_dir) {
        Ok(b) => b,
        Err(CppCompileError::Cxx(err)) => {
            return (
                id,
                CaseOutcome {
                    status: "cxx_err".into(),
                    detail: err,
                },
            );
        }
        Err(CppCompileError::Infra(err)) => {
            return (
                id,
                CaseOutcome {
                    status: "infra".into(),
                    detail: err,
                },
            );
        }
    };

    for ((inp_name, inp), interp_out) in inputs.iter().zip(interp_outputs.iter()) {
        let cpp_out = match run_binary(&bin, inp) {
            Ok(o) => o,
            Err(err) => {
                return (
                    id,
                    CaseOutcome {
                        status: "cpp_trap".into(),
                        detail: if inp_name.is_empty() {
                            err
                        } else {
                            format!("{inp_name}: {err}")
                        },
                    },
                );
            }
        };
        let interp_s = String::from_utf8_lossy(interp_out);
        let cpp_s = String::from_utf8_lossy(&cpp_out);
        let equal = if token_compare {
            interp_s.split_whitespace().eq(cpp_s.split_whitespace())
        } else {
            interp_s.trim_end_matches('\n') == cpp_s.trim_end_matches('\n')
        };
        if !equal {
            return (
                id,
                CaseOutcome {
                    status: "mismatch".into(),
                    detail: format!(
                        "{}interp={:?} cpp={:?}",
                        if inp_name.is_empty() {
                            String::new()
                        } else {
                            format!("{inp_name}: ")
                        },
                        runner::truncate_chars(interp_s.trim(), 80),
                        runner::truncate_chars(cpp_s.trim(), 80),
                    ),
                },
            );
        }
    }

    (
        id,
        CaseOutcome {
            status: "pass".into(),
            detail: String::new(),
        },
    )
}

pub fn run(root: &Path, filter: Option<&str>) -> SuiteReport {
    let targets = discover(root, filter);
    // The per-user g++ cache (scripts/check.sh, kept across CI runs) when there is one,
    // so a fresh checkout or CI job doesn't recompile every case.
    let cache_dir = match std::env::var_os("CPL_CXX_CACHE_DIR") {
        Some(dir) => PathBuf::from(dir).join("conform-diff"),
        None => std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("target"))
            .join("conform-cxx"),
    };

    let outcomes = runner::parallel_with_db(&targets, |db, target| {
        let (id, outcome) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_one(db, &cache_dir, target)
        }))
        .unwrap_or_else(|_| {
            let id = match target {
                DiffTarget::Conformance { id, .. }
                | DiffTarget::Corpus { id, .. }
                | DiffTarget::Variant { id, .. } => id.clone(),
            };
            (
                id,
                CaseOutcome {
                    status: "panic".into(),
                    detail: "codegen/interp panicked".into(),
                },
            )
        });
        (id, outcome)
    });
    SuiteReport::from_outcomes(outcomes)
}
