//! Zero-cost abstraction benchmark runner (`cpl bench --zero-cost`).
//!
//! For each subdirectory of `bench/zero-cost/` containing `abstract.cp` and
//! `direct.cp` (plus `input.txt` or `gen.py`), emits C++17, builds both with
//! `g++ -O2 -std=c++17`, verifies identical stdout, runs `reps` alternating
//! iterations, compares the fastest wall time of each (`abstract / direct <=
//! max_ratio`; noise from a loaded machine only adds time, so the minimum is
//! the stable estimate for a deterministic program),
//! and compares normalized `-O2 -S` assembly of the `kernel` function.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use cpl_diag::Shown;
use cpl_json::escape_string as json_string;

pub const DEFAULT_DIR: &str = "bench/zero-cost";
pub const DEFAULT_REPS: usize = 5;
pub const DEFAULT_MAX_RATIO: f64 = 1.05;

#[derive(Debug, Clone)]
pub struct PairResult {
    pub name: String,
    pub abstract_ms: f64,
    pub direct_ms: f64,
    pub cpp_ms: Option<f64>,
    pub ratio: f64,
    pub cpp_ratio: Option<f64>,
    pub asm_equal: Option<bool>,
    pub passed: bool,
    pub error: Option<String>,
}

/// Emits C++17 source for a `.cp` file using `cpl-check` + `cpl-ir` + `cpl-codegen`.
pub fn emit_cpp(path: &Path) -> Result<String, String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.shown()))?;
    let mut db = cpl_check::Database::new();
    let name = path.to_string_lossy().into_owned();
    db.set_file(&name, source.as_str());
    let analysis = db
        .analyze(&name)
        .ok_or_else(|| format!("{}: analysis returned none", path.shown()))?;
    if analysis.has_errors() {
        let msgs: Vec<String> = analysis
            .diagnostics
            .iter()
            .filter(|d| d.severity == cpl_diag::Severity::Error)
            .map(|d| d.message.clone())
            .collect();
        return Err(format!("{}: {}", path.shown(), msgs.join("; ")));
    }
    let ir = cpl_ir::lower(&analysis).map_err(|diags| {
        let msgs: Vec<String> = diags.into_iter().map(|d| d.message).collect();
        format!("{}: {}", path.shown(), msgs.join("; "))
    })?;
    Ok(cpl_codegen::emit(&ir, &name, Some(&source)))
}

fn compile_cpp(cpp_path: &Path, bin_path: &Path) -> Result<(), String> {
    let out = Command::new("g++")
        .args(["-O2", "-std=c++17"])
        .arg(cpp_path)
        .arg("-o")
        .arg(bin_path)
        .output()
        .map_err(|e| format!("can't run g++: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "g++ failed on {}:\n{}",
            cpp_path.shown(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(())
}

fn compile_asm(cpp_path: &Path, asm_path: &Path) -> Result<String, String> {
    let out = Command::new("g++")
        .args(["-O2", "-std=c++17", "-S"])
        .arg(cpp_path)
        .arg("-o")
        .arg(asm_path)
        .output()
        .map_err(|e| format!("can't run g++ -S: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    std::fs::read_to_string(asm_path).map_err(|e| e.to_string())
}

/// Extracts and normalizes the hot `kernel` function assembly (falling back to
/// `solve` when `g++ -O2` inlines `kernel`) for an informational equality check
/// between `abstract.cp` and `direct.cp`.
fn normalized_kernel_asm(asm: &str) -> Option<String> {
    extract_fn_asm(asm, "kernel").or_else(|| extract_fn_asm(asm, "solve"))
}

fn extract_fn_asm(asm: &str, needle: &str) -> Option<String> {
    let mut lines = Vec::new();
    let mut inside = false;
    for raw in asm.lines() {
        let line = raw.trim();
        if !inside {
            if line.ends_with(':') && line.contains(needle) && !line.starts_with('.') {
                inside = true;
            }
            continue;
        }
        if line.starts_with(".size") || (line.ends_with(':') && !line.starts_with('.')) {
            break;
        }
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("//")
            || line.starts_with(".cfi_")
            || line.starts_with(".loc")
            || line.starts_with(".file")
            || line.starts_with(".p2align")
        {
            continue;
        }
        let code = line.split('#').next().unwrap_or(line).trim();
        if code.is_empty() {
            continue;
        }
        lines.push(normalize_asm_line(code));
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn normalize_asm_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1] == b'L' {
            let mut j = i + 2;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            out.push_str(".L_");
            i = j;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn generate_input(dir: &Path, input_path: &Path) -> Result<(), String> {
    let static_in = dir.join("input.txt");
    if static_in.is_file() {
        std::fs::copy(&static_in, input_path)
            .map_err(|e| format!("copy {}: {e}", static_in.shown()))?;
        return Ok(());
    }
    let gen_py = dir.join("gen.py");
    if gen_py.is_file() {
        let out = Command::new("python3")
            .arg(&gen_py)
            .output()
            .map_err(|e| format!("can't run python3 {}: {e}", gen_py.shown()))?;
        if !out.status.success() {
            return Err(format!(
                "gen.py failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        std::fs::write(input_path, &out.stdout).map_err(|e| format!("write input: {e}"))?;
        return Ok(());
    }
    std::fs::write(input_path, b"").map_err(|e| e.to_string())
}

fn run_binary(bin: &Path, input_path: &Path) -> Result<(f64, Vec<u8>), String> {
    let file =
        std::fs::File::open(input_path).map_err(|e| format!("open {}: {e}", input_path.shown()))?;
    let start = Instant::now();
    let out = Command::new(bin)
        .stdin(Stdio::from(file))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("run {}: {e}", bin.shown()))?;
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    if !out.status.success() {
        return Err(format!(
            "{} exited with {}: {}",
            bin.shown(),
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok((elapsed_ms, out.stdout))
}

fn fastest(values: &[f64]) -> f64 {
    values.iter().copied().reduce(f64::min).unwrap_or(0.0)
}

pub fn run_pair(dir: &Path, reps: usize, max_ratio: f64) -> PairResult {
    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    let scratch = std::env::temp_dir().join(format!(
        "cpl-zero-cost-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::create_dir_all(&scratch);

    let res = (|| -> Result<PairResult, String> {
        let abs_cp = dir.join("abstract.cp");
        let dir_cp = dir.join("direct.cp");
        let hand_cpp_src = dir.join("hand.cpp");
        let has_hand_cpp = hand_cpp_src.is_file();
        let abs_cpp = scratch.join("abstract.cpp");
        let dir_cpp = scratch.join("direct.cpp");
        let abs_bin = scratch.join("abstract.bin");
        let dir_bin = scratch.join("direct.bin");
        let hand_bin = scratch.join("hand.bin");
        let input_path = scratch.join("input.txt");

        std::fs::write(&abs_cpp, emit_cpp(&abs_cp)?).map_err(|e| e.to_string())?;
        std::fs::write(&dir_cpp, emit_cpp(&dir_cp)?).map_err(|e| e.to_string())?;
        compile_cpp(&abs_cpp, &abs_bin)?;
        compile_cpp(&dir_cpp, &dir_bin)?;
        if has_hand_cpp {
            compile_cpp(&hand_cpp_src, &hand_bin)?;
        }
        generate_input(dir, &input_path)?;

        let asm_equal = match (
            compile_asm(&abs_cpp, &scratch.join("abstract.s")),
            compile_asm(&dir_cpp, &scratch.join("direct.s")),
        ) {
            (Ok(a), Ok(b)) => match (normalized_kernel_asm(&a), normalized_kernel_asm(&b)) {
                (Some(ka), Some(kb)) => Some(ka == kb),
                _ => None,
            },
            _ => None,
        };

        let mut abs_times = Vec::with_capacity(reps);
        let mut dir_times = Vec::with_capacity(reps);
        let mut hand_times = Vec::with_capacity(reps);
        let mut expected_out: Option<Vec<u8>> = None;

        for r in 0..reps {
            let (ta, oa) = run_binary(&abs_bin, &input_path)?;
            let (td, od) = run_binary(&dir_bin, &input_path)?;
            let hand_run = if has_hand_cpp {
                Some(run_binary(&hand_bin, &input_path)?)
            } else {
                None
            };
            if r == 0 {
                if oa != od {
                    return Err(format!(
                        "stdout mismatch: abstract={:?} direct={:?}",
                        String::from_utf8_lossy(&oa).trim(),
                        String::from_utf8_lossy(&od).trim()
                    ));
                }
                if let Some((_, ref oh)) = hand_run {
                    if &oa != oh {
                        return Err(format!(
                            "stdout mismatch: abstract={:?} hand.cpp={:?}",
                            String::from_utf8_lossy(&oa).trim(),
                            String::from_utf8_lossy(oh).trim()
                        ));
                    }
                }
                expected_out = Some(oa);
            } else if expected_out.as_ref() != Some(&oa)
                || expected_out.as_ref() != Some(&od)
                || hand_run
                    .as_ref()
                    .is_some_and(|(_, oh)| expected_out.as_ref() != Some(oh))
            {
                return Err("non-deterministic stdout across repetitions".to_string());
            }
            abs_times.push(ta);
            dir_times.push(td);
            if let Some((th, _)) = hand_run {
                hand_times.push(th);
            }
        }

        let abstract_ms = fastest(&abs_times);
        let direct_ms = fastest(&dir_times);
        let cpp_ms = if has_hand_cpp {
            Some(fastest(&hand_times))
        } else {
            None
        };
        let ratio = if direct_ms > 0.0 {
            abstract_ms / direct_ms
        } else {
            1.0
        };
        let cpp_ratio = cpp_ms.map(|c| if c > 0.0 { abstract_ms / c } else { 1.0 });
        let passed = ratio <= max_ratio && cpp_ratio.is_none_or(|cr| cr <= max_ratio);
        Ok(PairResult {
            name: name.clone(),
            abstract_ms,
            direct_ms,
            cpp_ms,
            ratio,
            cpp_ratio,
            asm_equal,
            passed,
            error: None,
        })
    })();

    let _ = std::fs::remove_dir_all(&scratch);
    match res {
        Ok(r) => r,
        Err(msg) => PairResult {
            name,
            abstract_ms: 0.0,
            direct_ms: 0.0,
            cpp_ms: None,
            ratio: f64::INFINITY,
            cpp_ratio: None,
            asm_equal: None,
            passed: false,
            error: Some(msg),
        },
    }
}

pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let mut dir = PathBuf::from(DEFAULT_DIR);
    let mut reps = DEFAULT_REPS;
    let mut max_ratio = DEFAULT_MAX_RATIO;
    let mut as_json = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--zero-cost" => {}
            "--reps" | "--runs" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    Some(n) if n > 0 => reps = n,
                    _ => {
                        let _ = writeln!(
                            stderr,
                            "cpl bench --zero-cost: --reps expects a positive integer"
                        );
                        return 2;
                    }
                }
            }
            "--max-ratio" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse::<f64>().ok()) {
                    Some(r) if r > 0.0 => max_ratio = r,
                    _ => {
                        let _ = writeln!(
                            stderr,
                            "cpl bench --zero-cost: --max-ratio expects a positive number"
                        );
                        return 2;
                    }
                }
            }
            "--format" => {
                i += 1;
                match args.get(i).map(String::as_str) {
                    Some("json") => as_json = true,
                    Some("text") => as_json = false,
                    _ => {
                        let _ = writeln!(
                            stderr,
                            "cpl bench --zero-cost: --format must be `text` or `json`"
                        );
                        return 2;
                    }
                }
            }
            arg if !arg.starts_with('-') => dir = PathBuf::from(arg),
            other => {
                let _ = writeln!(stderr, "cpl bench --zero-cost: unknown option `{other}`");
                return 2;
            }
        }
        i += 1;
    }

    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => {
            let _ = writeln!(
                stderr,
                "cpl bench --zero-cost: can't read {}: {e}",
                dir.shown()
            );
            return 2;
        }
    };
    let mut pairs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("abstract.cp").is_file() && p.join("direct.cp").is_file())
        .collect();
    pairs.sort();
    if pairs.is_empty() {
        let _ = writeln!(
            stderr,
            "cpl bench --zero-cost: no abstract.cp/direct.cp pairs found under {}",
            dir.shown()
        );
        return 2;
    }

    let results: Vec<PairResult> = pairs.iter().map(|p| run_pair(p, reps, max_ratio)).collect();
    let all_pass = results.iter().all(|r| r.passed);

    if as_json {
        let rows: Vec<String> = results
            .iter()
            .map(|r| {
                let asm = match r.asm_equal {
                    Some(true) => "\"equal\"",
                    Some(false) => "\"different\"",
                    None => "null",
                };
                let cpp_ms = r
                    .cpp_ms
                    .map_or("null".to_string(), |v| format!("{v:.3}"));
                let cpp_ratio = r
                    .cpp_ratio
                    .map_or("null".to_string(), |v| format!("{v:.4}"));
                format!(
                    "{{\"name\":{},\"abstract_ms\":{:.3},\"direct_ms\":{:.3},\"cpp_ms\":{},\"ratio\":{:.4},\"cpp_ratio\":{},\"asm\":{},\"passed\":{},\"error\":{}}}",
                    json_string(&r.name),
                    r.abstract_ms,
                    r.direct_ms,
                    cpp_ms,
                    if r.ratio.is_finite() { r.ratio } else { 0.0 },
                    cpp_ratio,
                    asm,
                    r.passed,
                    r.error.as_deref().map_or("null".to_string(), json_string),
                )
            })
            .collect();
        let _ = writeln!(
            stdout,
            "{{\"status\":{},\"reps\":{},\"max_ratio\":{:.2},\"pairs\":[{}]}}",
            json_string(if all_pass { "ok" } else { "fail" }),
            reps,
            max_ratio,
            rows.join(",")
        );
    } else {
        let _ = writeln!(
            stdout,
            "Zero-cost abstraction benchmark (fastest of {reps} runs, bar <= {max_ratio:.2}x):"
        );
        let _ = writeln!(
            stdout,
            "  {:<16} {:>10} {:>10} {:>8} {:>10} {:>8} {:>10}  status",
            "abstraction", "abstract", "direct", "vs_dir", "hand.cpp", "vs_cpp", "asm(-O2)"
        );
        for r in &results {
            if let Some(err) = &r.error {
                let _ = writeln!(stdout, "  {:<16} ERROR: {err}", r.name);
            } else {
                let asm = match r.asm_equal {
                    Some(true) => "equal",
                    Some(false) => "different",
                    None => "n/a",
                };
                let cpp_col = r
                    .cpp_ms
                    .map_or_else(|| "n/a".to_string(), |v| format!("{v:>8.2}ms"));
                let cpp_ratio_col = r
                    .cpp_ratio
                    .map_or_else(|| "n/a".to_string(), |v| format!("{v:>7.3}x"));
                let _ = writeln!(
                    stdout,
                    "  {:<16} {:>8.2}ms {:>8.2}ms {:>7.3}x {:>10} {:>8} {:>10}  {}",
                    r.name,
                    r.abstract_ms,
                    r.direct_ms,
                    r.ratio,
                    cpp_col,
                    cpp_ratio_col,
                    asm,
                    if r.passed { "PASS" } else { "FAIL" }
                );
            }
        }
    }

    if all_pass {
        0
    } else {
        1
    }
}
