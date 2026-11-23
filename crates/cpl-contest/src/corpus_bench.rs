//! Corpus-wide max-N runtime benchmark (`cpl bench --corpus`).
//!
//! Compares every measurable problem under `corpus/problems/<id>/` against its
//! original accepted C++ solution (both built with `g++ -O2 -std=c++17`),
//! reports per-port ratio, p50 and p95, enforces:
//!   - corpus p50 <= 1.1x, p95 <= 1.2x, max <= 1.5x (over ports above the 10 ms floor)
//!   - when `perf.cp` exists beside `spec.cp`/`port.cp`, uses it as the tuned
//!     variant, verifies it passes the problem's tests, and requires <= 1.1x
//!
//! Writes `research/perf/corpus-p95-results.json` in the schema `web/lib/bench.ts`
//! consumes.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cpl_json::escape_string as json_string;

use crate::zero_cost::emit_cpp;
use cpl_diag::Shown;

const DEFAULT_CORPUS_DIR: &str = "corpus/problems";
const DEFAULT_OUT_JSON: &str = "research/perf/corpus-p95-results.json";
const DEFAULT_RUNS: usize = 5;
const FLOOR_S: f64 = 0.010;
const BAR_P50: f64 = 1.1;
const BAR_P95: f64 = 1.2;
const BAR_MAX: f64 = 1.5;
const BAR_TUNED: f64 = 1.1;
const SIZE_LIMIT: usize = 64 * 1024;

#[derive(Debug, Clone)]
struct PortRecord {
    problem: String,
    port: String,
    tuned: bool,
    emitted_bytes: usize,
    input: Option<String>,
    orig_s: Option<f64>,
    port_s: Option<f64>,
    orig_rss_kb: Option<u64>,
    port_rss_kb: Option<u64>,
    time_ratio: Option<f64>,
    rss_ratio: Option<f64>,
    above_floor: bool,
    skip: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct Dist {
    n: usize,
    median: f64,
    p90: f64,
    p95: f64,
    p99: f64,
    worst: f64,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn summarize(mut vals: Vec<f64>) -> Dist {
    if vals.is_empty() {
        return Dist::default();
    }
    vals.sort_by(|a, b| a.total_cmp(b));
    let n = vals.len();
    let median = if n % 2 == 1 {
        vals[n / 2]
    } else {
        0.5 * (vals[n / 2 - 1] + vals[n / 2])
    };
    Dist {
        n,
        median,
        p90: percentile(&vals, 0.90),
        p95: percentile(&vals, 0.95),
        p99: percentile(&vals, 0.99),
        worst: vals[n - 1],
    }
}

fn median_f64(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.total_cmp(b));
    let n = vals.len();
    if n == 0 {
        0.0
    } else if n % 2 == 1 {
        vals[n / 2]
    } else {
        0.5 * (vals[n / 2 - 1] + vals[n / 2])
    }
}

fn find_orig_cpp(prob_dir: &Path) -> Option<PathBuf> {
    let direct = prob_dir.join("orig.cpp");
    if direct.is_file() {
        return Some(direct);
    }
    // Fall back to `submissions[*].file` in meta.json.
    let meta = std::fs::read_to_string(prob_dir.join("meta.json")).ok()?;
    let needle = "\"file\": \"";
    let mut rest = meta.as_str();
    while let Some(pos) = rest.find(needle) {
        let after = &rest[pos + needle.len()..];
        if let Some(end) = after.find('"') {
            let rel = &after[..end];
            if rel.ends_with(".cpp") {
                let cand = PathBuf::from(rel);
                if cand.is_file() {
                    return Some(cand);
                }
            }
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    None
}

fn choose_port(prob_dir: &Path) -> Option<(PathBuf, bool)> {
    let perf = prob_dir.join("perf.cp");
    if perf.is_file() {
        return Some((perf, true));
    }
    let port = prob_dir.join("port.cp");
    if port.is_file() {
        return Some((port, false));
    }
    let spec = prob_dir.join("spec.cp");
    if spec.is_file() {
        return Some((spec, false));
    }
    None
}

fn find_input(prob_dir: &Path, port_path: &Path, scratch: &Path) -> Option<(PathBuf, String)> {
    let gen_py = prob_dir.join("gen.py");
    if gen_py.is_file() {
        let out_path = scratch.join("gen.in");
        let out = Command::new("python3")
            .arg(&gen_py)
            .current_dir(prob_dir)
            .output()
            .ok()?;
        if out.status.success()
            && !out.stdout.is_empty()
            && std::fs::write(&out_path, &out.stdout).is_ok()
        {
            return Some((out_path, "gen.py".to_string()));
        }
    }
    let tests_dir = prob_dir.join("tests");
    if tests_dir.is_dir() {
        let mut best: Option<(u64, PathBuf)> = None;
        if let Ok(rd) = std::fs::read_dir(&tests_dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("in") {
                    if let Ok(m) = p.metadata() {
                        if best.as_ref().is_none_or(|(sz, _)| m.len() > *sz) {
                            best = Some((m.len(), p));
                        }
                    }
                }
            }
        }
        if let Some((_, p)) = best {
            let label = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("test.in")
                .to_string();
            return Some((p, format!("tests/{label}")));
        }
    }
    if let Ok(source) = std::fs::read_to_string(port_path) {
        if let Ok(spec) = cpl_stress::InputSpec::from_source(&source) {
            if !spec.is_empty() {
                if let Some(case) =
                    crate::bench::worst_case(&spec, crate::bench::WorstShape::Random, 1)
                {
                    let rendered = case.render();
                    if !rendered.is_empty() {
                        let out_path = scratch.join("worst.in");
                        if std::fs::write(&out_path, rendered.as_bytes()).is_ok() {
                            return Some((out_path, "derived/random".to_string()));
                        }
                    }
                }
            }
        }
    }
    None
}

fn compile_cpp(src: &Path, bin: &Path) -> Result<(), String> {
    let out = Command::new("g++")
        .args(["-O2", "-std=c++17"])
        .arg(src)
        .arg("-o")
        .arg(bin)
        .output()
        .map_err(|e| format!("can't run g++: {e}"))?;
    if !out.status.success() {
        return Err("g++ build failed".to_string());
    }
    Ok(())
}

fn run_bin(bin: &Path, input: &Path) -> Result<(f64, Vec<u8>), String> {
    let f = std::fs::File::open(input).map_err(|e| e.to_string())?;
    let start = Instant::now();
    let mut child = Command::new(bin)
        .stdin(Stdio::from(f))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut out_reader = child.stdout.take();
    let reader_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(ref mut r) = out_reader {
            let _ = std::io::Read::read_to_end(r, &mut buf);
        }
        buf
    });
    let timeout = Duration::from_secs(10);
    let status = loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(s) => break s,
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader_handle.join();
                return Err("timeout".to_string());
            }
            None => std::thread::sleep(Duration::from_micros(200)),
        }
    };
    let secs = start.elapsed().as_secs_f64();
    let stdout = reader_handle.join().unwrap_or_default();
    if !status.success() {
        return Err(format!("exit {status}"));
    }
    Ok((secs, stdout))
}

fn tokens_equal(a: &[u8], b: &[u8]) -> bool {
    let sa = String::from_utf8_lossy(a);
    let sb = String::from_utf8_lossy(b);
    let mut ia = sa.split_whitespace();
    let mut ib = sb.split_whitespace();
    loop {
        match (ia.next(), ib.next()) {
            (None, None) => return true,
            (Some(x), Some(y)) => {
                if x == y {
                    continue;
                }
                match (x.parse::<f64>(), y.parse::<f64>()) {
                    (Ok(fx), Ok(fy))
                        if (fx - fy).abs() <= 1e-6 * fx.abs().max(fy.abs()).max(1.0) => {}
                    _ => return false,
                }
            }
            _ => return false,
        }
    }
}

/// Verifies `perf.cp` passes every `tests/*.in` + `.out`/`.ans` case.
fn verify_perf_tests(bin: &Path, prob_dir: &Path) -> Result<(), String> {
    let tests_dir = prob_dir.join("tests");
    if !tests_dir.is_dir() {
        return Ok(());
    }
    let mut ins: Vec<PathBuf> = std::fs::read_dir(&tests_dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("in"))
        .collect();
    ins.sort();
    for inp in ins {
        let out_file = inp.with_extension("out");
        let ans_file = inp.with_extension("ans");
        let expected_path = if out_file.is_file() {
            out_file
        } else if ans_file.is_file() {
            ans_file
        } else {
            continue;
        };
        let expected = std::fs::read(&expected_path).map_err(|e| e.to_string())?;
        let (_, got) =
            run_bin(bin, &inp).map_err(|e| format!("perf.cp failed on {}: {e}", inp.shown()))?;
        if !tokens_equal(&got, &expected) {
            return Err(format!("perf.cp WA on {}", inp.shown()));
        }
    }
    Ok(())
}

fn bench_problem(prob_dir: &Path, runs: usize) -> PortRecord {
    let problem = prob_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    let Some((port_path, tuned)) = choose_port(prob_dir) else {
        return PortRecord {
            problem,
            port: String::new(),
            tuned: false,
            emitted_bytes: 0,
            input: None,
            orig_s: None,
            port_s: None,
            orig_rss_kb: None,
            port_rss_kb: None,
            time_ratio: None,
            rss_ratio: None,
            above_floor: false,
            skip: Some("no port.cp/spec.cp".to_string()),
        };
    };
    let port_str = port_path.to_string_lossy().into_owned();
    let Some(orig_cpp) = find_orig_cpp(prob_dir) else {
        return PortRecord {
            problem,
            port: port_str,
            tuned,
            emitted_bytes: 0,
            input: None,
            orig_s: None,
            port_s: None,
            orig_rss_kb: None,
            port_rss_kb: None,
            time_ratio: None,
            rss_ratio: None,
            above_floor: false,
            skip: Some("no original C++".to_string()),
        };
    };

    let scratch = std::env::temp_dir().join(format!(
        "cpl-corpus-bench-{problem}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::create_dir_all(&scratch);

    let result = (|| -> Result<PortRecord, (usize, String)> {
        let emitted = emit_cpp(&port_path).map_err(|e| (0, e))?;
        let emitted_bytes = emitted.len();
        let port_cpp = scratch.join("port.cpp");
        std::fs::write(&port_cpp, &emitted).map_err(|e| (emitted_bytes, e.to_string()))?;

        let port_bin = scratch.join("port.bin");
        let orig_bin = scratch.join("orig.bin");
        compile_cpp(&port_cpp, &port_bin).map_err(|e| (emitted_bytes, format!("port {e}")))?;
        compile_cpp(&orig_cpp, &orig_bin).map_err(|e| (emitted_bytes, format!("orig {e}")))?;

        if tuned {
            verify_perf_tests(&port_bin, prob_dir).map_err(|e| (emitted_bytes, e))?;
        }

        let (inp_path, inp_label) = find_input(prob_dir, &port_path, &scratch)
            .ok_or_else(|| (emitted_bytes, "no max-N input".to_string()))?;

        let mut orig_times = Vec::with_capacity(runs);
        let mut port_times = Vec::with_capacity(runs);
        for r in 0..runs {
            let (to, oo) = run_bin(&orig_bin, &inp_path)
                .map_err(|e| (emitted_bytes, format!("orig run: {e}")))?;
            let (tp, op) = run_bin(&port_bin, &inp_path)
                .map_err(|e| (emitted_bytes, format!("port run: {e}")))?;
            if r == 0 && !tokens_equal(&oo, &op) {
                return Err((emitted_bytes, "output mismatch vs orig.cpp".to_string()));
            }
            orig_times.push(to);
            port_times.push(tp);
        }

        let orig_s = median_f64(&mut orig_times);
        let port_s = median_f64(&mut port_times);
        let time_ratio = if orig_s > 0.0 { port_s / orig_s } else { 1.0 };
        let above_floor = orig_s >= FLOOR_S || port_s >= FLOOR_S;

        Ok(PortRecord {
            problem: problem.clone(),
            port: port_str.clone(),
            tuned,
            emitted_bytes,
            input: Some(inp_label),
            orig_s: Some(orig_s),
            port_s: Some(port_s),
            orig_rss_kb: Some(0),
            port_rss_kb: Some(0),
            time_ratio: Some(time_ratio),
            rss_ratio: Some(1.0),
            above_floor,
            skip: None,
        })
    })();

    let _ = std::fs::remove_dir_all(&scratch);
    match result {
        Ok(rec) => rec,
        Err((emitted_bytes, reason)) => PortRecord {
            problem,
            port: port_str,
            tuned,
            emitted_bytes,
            input: None,
            orig_s: None,
            port_s: None,
            orig_rss_kb: None,
            port_rss_kb: None,
            time_ratio: None,
            rss_ratio: None,
            above_floor: false,
            skip: Some(reason),
        },
    }
}

fn gxx_version() -> String {
    Command::new("g++")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(str::trim).map(ToString::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn git_short_commit() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn dist_json(d: &Dist) -> String {
    format!(
        "{{\"n\":{},\"median\":{:.3},\"p90\":{:.3},\"p95\":{:.3},\"p99\":{:.3},\"worst\":{:.3}}}",
        d.n, d.median, d.p90, d.p95, d.p99, d.worst
    )
}

fn build_json(
    records: &[PortRecord],
    runs: usize,
    total_problems: usize,
    target_met: bool,
) -> String {
    let measured: Vec<&PortRecord> = records.iter().filter(|r| r.time_ratio.is_some()).collect();
    let above: Vec<&PortRecord> = measured.iter().copied().filter(|r| r.above_floor).collect();
    let time_all = summarize(measured.iter().filter_map(|r| r.time_ratio).collect());
    let time_above = summarize(above.iter().filter_map(|r| r.time_ratio).collect());
    let rss_all = summarize(measured.iter().filter_map(|r| r.rss_ratio).collect());
    let rss_above = summarize(above.iter().filter_map(|r| r.rss_ratio).collect());

    let emitted_files = records.iter().filter(|r| r.emitted_bytes > 0).count();
    let largest = records
        .iter()
        .max_by_key(|r| r.emitted_bytes)
        .map(|r| (r.emitted_bytes, r.port.as_str()))
        .unwrap_or((0, ""));
    let over_64k: Vec<String> = records
        .iter()
        .filter(|r| r.emitted_bytes > SIZE_LIMIT)
        .map(|r| json_string(&r.port))
        .collect();

    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    let ports_json: Vec<String> = records
        .iter()
        .map(|r| {
            let mut fields = vec![
                format!("\"problem\":{}", json_string(&r.problem)),
                format!("\"port\":{}", json_string(&r.port)),
                format!("\"emitted_bytes\":{}", r.emitted_bytes),
            ];
            if r.tuned {
                fields.push("\"tuned\":true".to_string());
            }
            if let Some(inp) = &r.input {
                fields.push(format!("\"input\":{}", json_string(inp)));
            }
            if let Some(v) = r.orig_s {
                fields.push(format!("\"orig_s\":{v:.6}"));
            }
            if let Some(v) = r.port_s {
                fields.push(format!("\"port_s\":{v:.6}"));
            }
            if let Some(v) = r.orig_rss_kb {
                fields.push(format!("\"orig_rss_kb\":{v}"));
            }
            if let Some(v) = r.port_rss_kb {
                fields.push(format!("\"port_rss_kb\":{v}"));
            }
            if let Some(v) = r.time_ratio {
                fields.push(format!("\"time_ratio\":{v:.4}"));
                fields.push(format!("\"above_floor\":{}", r.above_floor));
            }
            if let Some(v) = r.rss_ratio {
                fields.push(format!("\"rss_ratio\":{v:.4}"));
            }
            if let Some(sk) = &r.skip {
                fields.push(format!("\"skip\":{}", json_string(sk)));
            }
            format!("{{{}}}", fields.join(","))
        })
        .collect();

    format!(
        "{{\n  \"meta\": {{\"commit\":{},\"cpl_version\":{},\"gxx\":{},\"runs\":{},\"problems\":{},\"cpu\":{},\"cores\":{}}},\n  \"summary\": {{\"measured\":{},\"above_floor\":{},\"floor_ms\":{:.1},\"max_emitted_bytes\":{},\"emitted_files\":{},\"emitted_over_64kb\":[{}],\"largest_emitted_port\":{},\"time_all\":{},\"rss_all\":{},\"time_above_floor\":{},\"rss_above_floor\":{},\"target_met\":{}}},\n  \"ports\": [\n    {}\n  ]\n}}\n",
        json_string(&git_short_commit()),
        json_string(env!("CARGO_PKG_VERSION")),
        json_string(&gxx_version()),
        runs,
        total_problems,
        json_string(std::env::consts::ARCH),
        cores,
        measured.len(),
        above.len(),
        FLOOR_S * 1000.0,
        largest.0,
        emitted_files,
        over_64k.join(","),
        json_string(largest.1),
        dist_json(&time_all),
        dist_json(&rss_all),
        dist_json(&time_above),
        dist_json(&rss_above),
        target_met,
        ports_json.join(",\n    "),
    )
}

pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let mut corpus_dir = PathBuf::from(DEFAULT_CORPUS_DIR);
    let mut out_json = PathBuf::from(DEFAULT_OUT_JSON);
    let mut runs = DEFAULT_RUNS;
    let mut no_write = false;
    let mut filter_ports: Option<Vec<String>> = None;
    let mut limit: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus" => {}
            "--no-write" => no_write = true,
            "--runs" | "--reps" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    Some(n) if n > 0 => runs = n,
                    _ => {
                        let _ = writeln!(
                            stderr,
                            "cpl bench --corpus: --runs expects a positive integer"
                        );
                        return 2;
                    }
                }
            }
            "--ports" => {
                i += 1;
                let Some(list) = args.get(i) else {
                    let _ = writeln!(
                        stderr,
                        "cpl bench --corpus: --ports needs a comma-separated list"
                    );
                    return 2;
                };
                filter_ports = Some(
                    list.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                );
            }
            "--dir" => {
                i += 1;
                let Some(d) = args.get(i) else {
                    let _ = writeln!(stderr, "cpl bench --corpus: --dir needs a path");
                    return 2;
                };
                corpus_dir = PathBuf::from(d);
            }
            "--out" => {
                i += 1;
                let Some(o) = args.get(i) else {
                    let _ = writeln!(stderr, "cpl bench --corpus: --out needs a path");
                    return 2;
                };
                out_json = PathBuf::from(o);
            }
            "--limit" => {
                i += 1;
                limit = args.get(i).and_then(|s| s.parse::<usize>().ok());
            }
            other => {
                let message = cpl_diag::usage::no_option("bench", other);
                let _ = writeln!(stderr, "cpl: {message}");
                return 2;
            }
        }
        i += 1;
    }

    let entries = match std::fs::read_dir(&corpus_dir) {
        Ok(e) => e,
        Err(e) => {
            let _ = writeln!(
                stderr,
                "cpl bench --corpus: can't read {}: {e}",
                corpus_dir.shown()
            );
            return 2;
        }
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    let total_problems = dirs.len();

    if let Some(wanted) = &filter_ports {
        dirs.retain(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| wanted.iter().any(|w| w == n))
        });
    }
    if let Some(lim) = limit {
        dirs.truncate(lim);
    }

    let started = Instant::now();
    let mut records = Vec::with_capacity(dirs.len());
    for d in &dirs {
        let rec = bench_problem(d, runs);
        if let (Some(orig), Some(port), Some(ratio)) = (rec.orig_s, rec.port_s, rec.time_ratio) {
            let tag = if rec.tuned { " [perf.cp]" } else { "" };
            let _ = writeln!(
                stdout,
                "  {:<18} orig={:>7.1}ms  port={:>7.1}ms  ratio={:>6.3}x  {:>5}B{}",
                rec.problem,
                orig * 1000.0,
                port * 1000.0,
                ratio,
                rec.emitted_bytes,
                tag
            );
        }
        records.push(rec);
    }

    let above: Vec<f64> = records
        .iter()
        .filter(|r| r.above_floor)
        .filter_map(|r| r.time_ratio)
        .collect();
    let all_ratios: Vec<f64> = records.iter().filter_map(|r| r.time_ratio).collect();
    let dist = if !above.is_empty() {
        summarize(above)
    } else {
        summarize(all_ratios)
    };

    let mut tuned_failures: Vec<String> = Vec::new();
    for r in records.iter().filter(|r| r.tuned) {
        if let Some(sk) = &r.skip {
            tuned_failures.push(format!("{}: {sk}", r.problem));
        } else if let Some(ratio) = r.time_ratio {
            if ratio > BAR_TUNED {
                tuned_failures.push(format!("{}: {ratio:.3}x > {BAR_TUNED:.1}x", r.problem));
            }
        }
    }

    let bars_ok = dist.n > 0
        && dist.median <= BAR_P50
        && dist.p95 <= BAR_P95
        && dist.worst <= BAR_MAX
        && tuned_failures.is_empty();

    let _ = writeln!(
        stdout,
        "\nCorpus runtime summary ({} measured, {} above {:.0}ms floor, {:.1}s):",
        records.iter().filter(|r| r.time_ratio.is_some()).count(),
        dist.n,
        FLOOR_S * 1000.0,
        started.elapsed().as_secs_f64()
    );
    let _ = writeln!(
        stdout,
        "  p50={:.3}x (bar <= {BAR_P50:.1}x)  p90={:.3}x  p95={:.3}x (bar <= {BAR_P95:.1}x)  worst={:.3}x (bar <= {BAR_MAX:.1}x; perf.cp bar <= {BAR_TUNED:.1}x)",
        dist.median, dist.p90, dist.p95, dist.worst
    );
    if !tuned_failures.is_empty() {
        let _ = writeln!(
            stdout,
            "  perf.cp bar failures: {}",
            tuned_failures.join("; ")
        );
    }

    if !no_write {
        let json = build_json(&records, runs, total_problems, bars_ok);
        if let Err(e) = std::fs::write(&out_json, json) {
            let _ = writeln!(
                stderr,
                "cpl bench --corpus: can't write {}: {e}",
                out_json.shown()
            );
            return 1;
        }
        let _ = writeln!(stdout, "  wrote {}", out_json.shown());
    }

    // Keep Duration imported cleanly if needed later
    let _ = Duration::from_secs(0);
    if bars_ok {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod schema_tests {
    use super::*;
    use cpl_json::Value;
    use std::collections::BTreeMap;

    enum Ty {
        Num,
        Str,
        Bool,
        Named(String),
        Array(Box<Ty>),
        Object(Vec<(String, bool, Ty)>),
    }

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// Splits `body` on `;` and newlines outside braces.
    fn members(body: &str) -> Vec<&str> {
        let (mut depth, mut start, mut out) = (0, 0, vec![]);
        for (i, c) in body.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                ';' | '\n' if depth == 0 => {
                    out.push(body[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            }
        }
        out.push(body[start..].trim());
        out.into_iter().filter(|m| !m.is_empty()).collect()
    }

    fn ty(text: &str) -> Ty {
        let text = text.trim();
        if let Some(inner) = text.strip_suffix("[]") {
            return Ty::Array(Box::new(ty(inner)));
        }
        if let Some(body) = text.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
            return Ty::Object(
                members(body)
                    .into_iter()
                    .map(|m| {
                        let (name, t) = m.split_once(':').expect("`name: type`");
                        let (name, opt) = match name.trim().strip_suffix('?') {
                            Some(n) => (n, true),
                            None => (name.trim(), false),
                        };
                        (name.to_string(), opt, ty(t))
                    })
                    .collect(),
            );
        }
        match text {
            "number" => Ty::Num,
            "string" => Ty::Str,
            "boolean" => Ty::Bool,
            name => Ty::Named(name.to_string()),
        }
    }

    /// Every `export interface` in the site's reader of the results file.
    fn site_schema() -> BTreeMap<String, Ty> {
        let ts = std::fs::read_to_string(root().join("web/lib/bench.ts")).unwrap();
        let mut out = BTreeMap::new();
        for chunk in ts.split("export interface ").skip(1) {
            let (name, rest) = chunk.split_once('{').unwrap();
            let (mut depth, mut end) = (1, 0);
            for (i, c) in rest.char_indices() {
                depth += match c {
                    '{' => 1,
                    '}' => -1,
                    _ => 0,
                };
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            out.insert(
                name.trim().to_string(),
                ty(&format!("{{{}}}", &rest[..end])),
            );
        }
        out
    }

    fn check(v: &Value, t: &Ty, schema: &BTreeMap<String, Ty>, path: &str) -> Vec<String> {
        let ok = |good: bool| {
            if good {
                vec![]
            } else {
                vec![format!("{path}: wrong type")]
            }
        };
        match t {
            Ty::Num => ok(v.as_f64().is_some()),
            Ty::Str => ok(v.as_str().is_some()),
            Ty::Bool => ok(v.as_bool().is_some()),
            Ty::Named(n) => check(v, &schema[n], schema, path),
            Ty::Array(e) => match v.as_array() {
                Some(xs) => xs
                    .iter()
                    .enumerate()
                    .flat_map(|(i, x)| check(x, e, schema, &format!("{path}[{i}]")))
                    .collect(),
                None => ok(false),
            },
            Ty::Object(fields) => match v.as_object() {
                Some(o) => fields
                    .iter()
                    .flat_map(|(name, opt, ft)| match o.get(name) {
                        Some(fv) => check(fv, ft, schema, &format!("{path}.{name}")),
                        None if *opt => vec![],
                        None => vec![format!("{path}.{name}: missing")],
                    })
                    .collect(),
                None => ok(false),
            },
        }
    }

    fn record(problem: &str, measured: bool) -> PortRecord {
        PortRecord {
            problem: problem.into(),
            port: format!("corpus/problems/{problem}/port.cp"),
            tuned: measured,
            emitted_bytes: 1234,
            input: measured.then(|| "max-N".into()),
            orig_s: measured.then_some(0.02),
            port_s: measured.then_some(0.021),
            orig_rss_kb: measured.then_some(4000),
            port_rss_kb: measured.then_some(4100),
            time_ratio: measured.then_some(1.05),
            rss_ratio: measured.then_some(1.025),
            above_floor: measured,
            skip: (!measured).then(|| "orig run: \"timeout\"".into()),
        }
    }

    #[test]
    fn the_written_results_match_the_site_schema() {
        let schema = site_schema();
        assert!(schema.contains_key("Bench") && schema.contains_key("Port"));
        let json = build_json(&[record("a", true), record("b", false)], 3, 2, true);
        let v = cpl_json::parse(&json).unwrap();
        assert_eq!(
            check(&v, &schema["Bench"], &schema, "$"),
            Vec::<String>::new()
        );
        assert_eq!(v.at(&["ports"]).as_array().unwrap().len(), 2);
    }

    #[test]
    fn the_committed_results_match_the_site_schema() {
        let schema = site_schema();
        let text = std::fs::read_to_string(root().join(DEFAULT_OUT_JSON)).unwrap();
        let v = cpl_json::parse(&text).unwrap();
        let errors = check(&v, &schema["Bench"], &schema, "$");
        assert!(errors.is_empty(), "{:?}", &errors[..errors.len().min(10)]);
    }

    #[test]
    fn a_missing_field_breaks_the_schema() {
        let schema = site_schema();
        let json =
            build_json(&[record("a", true)], 3, 1, true).replace("\"emitted_bytes\"", "\"bytes\"");
        let errors = check(
            &cpl_json::parse(&json).unwrap(),
            &schema["Bench"],
            &schema,
            "$",
        );
        assert_eq!(
            errors,
            vec!["$.ports[0].emitted_bytes: missing".to_string()]
        );
    }
}
