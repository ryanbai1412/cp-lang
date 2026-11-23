use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use cpl_conform::conformance;
use cpl_conform::scoreboard::{self, Scoreboard, SuiteReport};
use cpl_conform::{default_scoreboard_path, find_repo_root, run_suites, Suite};

const USAGE: &str = "\
usage: cpl-conform [conformance|corpus|docs|spec-blocks|diff|rewrites|all]... [options] [filter]

suites:
  conformance     spec/conformance/**/*.cp cases
  corpus          corpus/problems/<id>/spec.cp on tests/*.in vs *.ans
  docs            /// example: lines in std/*.cp
  spec-blocks     fenced cp blocks in spec/core/*.md and spec/lib/*.md
  diff            interpreter vs g++ -O2 -std=c++17 differential, with
                  side-effect variants of corpus ports (variant/*)
  rewrites        corpus programs print the same after `cpl fix` (#fix)
                  and after `cpl fmt` (#fmt) on their tests
  all             all six suites (default if no suite is named)
  fuzz [START] [COUNT] [--no-cxx]
                  generated programs (seeds START..START+COUNT, default
                  0 200) on the interpreter and the C++ at every --checks
                  level; findings are kept in target/fuzz/<seed>.cp

options:
  --check         fail if any suite's pass count drops or a passing case regresses
  --write         update the scoreboard JSON with the suites that were run
  --shard N/M     diff runs only every Mth target, starting at the Nth
  --scoreboard P  scoreboard JSON path (default: research/scoreboard.json)
  --causes        group failing cases by root cause, sorted by count
  --rules         check spec/core rule coverage against spec/conformance
  --strict        fail on unmet conformance cases (or uncovered rules with --rules)
  --verbose       print failing case details
  -h, --help      show this help
";

fn format_counts(report: &SuiteReport) -> String {
    report
        .counts
        .iter()
        .map(|(k, v)| format!("{v} {k}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut suites: Vec<Suite> = Vec::new();
    let mut check = false;
    let mut write = false;
    let mut causes = false;
    let mut rules_only = false;
    let mut strict = false;
    let mut verbose = false;
    let mut scoreboard_path: Option<PathBuf> = None;
    let mut filter: Option<String> = None;

    if args.first().map(String::as_str) == Some("fuzz") {
        return fuzz(&args[1..]);
    }
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "--check" => check = true,
            "--write" => write = true,
            "--causes" => causes = true,
            "--rules" => rules_only = true,
            "--strict" => strict = true,
            "--verbose" | "-v" => verbose = true,
            "--shard" => {
                i += 1;
                let shard = args.get(i).and_then(|s| {
                    let (n, m) = s.split_once('/')?;
                    let (n, m) = (n.parse::<usize>().ok()?, m.parse::<usize>().ok()?);
                    (1..=m).contains(&n).then_some((n, m))
                });
                let Some(shard) = shard else {
                    eprintln!("cpl-conform: --shard needs N/M with 1 <= N <= M");
                    return ExitCode::from(2);
                };
                let _ = cpl_conform::runner::SHARD.set(shard);
            }
            "--scoreboard" | "--out" | "--baseline" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("cpl-conform: {arg} requires a path");
                    return ExitCode::from(2);
                }
                scoreboard_path = Some(PathBuf::from(&args[i]));
            }
            "all" => {
                for &s in Suite::ALL {
                    if !suites.contains(&s) {
                        suites.push(s);
                    }
                }
            }
            "fast" => {
                for &s in Suite::FAST {
                    if !suites.contains(&s) {
                        suites.push(s);
                    }
                }
            }
            other if Suite::parse(other).is_some() => {
                let s = Suite::parse(other).unwrap();
                if !suites.contains(&s) {
                    suites.push(s);
                }
            }
            other if other.starts_with('-') => {
                eprintln!("cpl-conform: unknown option `{other}`\n\n{USAGE}");
                return ExitCode::from(2);
            }
            other => {
                if filter.is_some() {
                    eprintln!("cpl-conform: unexpected argument `{other}`");
                    return ExitCode::from(2);
                }
                filter = Some(other.to_string());
            }
        }
        i += 1;
    }

    let root = match find_repo_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cpl-conform: {e}");
            return ExitCode::from(2);
        }
    };

    if rules_only {
        let cov = conformance::check_rules(&root);
        for (r, loc) in &cov.missing {
            println!("{loc}: R{r} has no conformance case");
        }
        for r in &cov.unknown {
            println!("a case cites R{r}, which no chapter defines");
        }
        let covered = cov.defined.len() - cov.missing.len();
        println!(
            "conformance rules: {covered} of {} covered, {} unknown",
            cov.defined.len(),
            cov.unknown.len()
        );
        if strict && (!cov.missing.is_empty() || !cov.unknown.is_empty()) {
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    if suites.is_empty() {
        suites.extend_from_slice(Suite::ALL);
    }

    let sb_path = scoreboard_path.unwrap_or_else(|| default_scoreboard_path(&root));
    let t0 = Instant::now();
    let current = run_suites(&root, &suites, filter.as_deref());
    let elapsed = t0.elapsed();

    let mut exit_fail = false;
    for &suite in &suites {
        let name = suite.name();
        let report = &current.suites[name];
        let strict_col = match report.strict_passed {
            Some(s) => format!(" [strict {s}/{}]", report.total),
            None => String::new(),
        };
        println!(
            "{name:12} {:>4}/{:<4} pass{strict_col} ({})",
            report.passed,
            report.total,
            format_counts(report)
        );
        if causes && !report.causes.is_empty() {
            println!("  causes ({} distinct):", report.causes.len());
            for entry in &report.causes {
                let ex = if entry.examples.is_empty() {
                    String::new()
                } else {
                    format!("  [{}]", entry.examples.join(", "))
                };
                println!("    {:>4}  {}{}", entry.count, entry.cause, ex);
            }
        }
        if verbose {
            for (case_id, status) in &report.cases {
                if status != "pass" && status != "untested" && status != "skip" {
                    let detail = report
                        .details
                        .get(case_id)
                        .map(String::as_str)
                        .unwrap_or("");
                    println!("  {case_id}: {status}: {detail}");
                }
            }
        }
        let bad = report.counts.get("bad").copied().unwrap_or(0);
        let panic = report.counts.get("panic").copied().unwrap_or(0);
        let infra = report.counts.get("infra").copied().unwrap_or(0);
        if bad > 0 || panic > 0 || infra > 0 {
            eprintln!(
                "cpl-conform: {name} has {bad} malformed, {panic} panicking, and {infra} infra-error cases"
            );
            exit_fail = true;
        }
        if strict && report.passed < report.total {
            exit_fail = true;
        }
    }
    println!("cpl-conform: finished in {:.2}s", elapsed.as_secs_f64());
    // `CPL_CONFORM_BUDGET` seconds (default 900): fail by name well before CI's job
    // timeout kills the run with no report.
    let budget: f64 = std::env::var("CPL_CONFORM_BUDGET")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(900.0);
    if elapsed.as_secs_f64() > budget {
        eprintln!("cpl-conform: over the {budget}s budget; slowest cases:");
        for (case, secs) in cpl_conform::runner::slowest_cases(10) {
            eprintln!("  {secs:7.2}s  {case}");
        }
        exit_fail = true;
    }

    if check {
        let baseline = match Scoreboard::load(&sb_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "cpl-conform: cannot load baseline `{}`: {e}",
                    sb_path.display()
                );
                return ExitCode::FAILURE;
            }
        };
        let ratchet = scoreboard::check_ratchet(
            &baseline,
            &current,
            filter.is_some() || cpl_conform::runner::SHARD.get().is_some(),
        );
        if !ratchet.improvements.is_empty() {
            println!(
                "ratchet: {} newly passing case(s)",
                ratchet.improvements.len()
            );
            for imp in ratchet.improvements.iter().take(20) {
                println!("  + {imp}");
            }
            if ratchet.improvements.len() > 20 {
                println!("  ... and {} more", ratchet.improvements.len() - 20);
            }
        }
        if !ratchet.regressions.is_empty() {
            eprintln!(
                "cpl-conform: RATCHET FAILED ({} regression(s)):",
                ratchet.regressions.len()
            );
            for reg in &ratchet.regressions {
                eprintln!("  - {reg}");
            }
            exit_fail = true;
        } else {
            println!("ratchet: ok (0 regressions against {})", sb_path.display());
        }
    }

    if write {
        if filter.is_some() {
            eprintln!("cpl-conform: refusing --write with a filter");
            return ExitCode::from(2);
        }
        let mut merged = Scoreboard::load(&sb_path).unwrap_or_default();
        merged.merge_from(&current);
        if let Err(e) = merged.save(&sb_path) {
            eprintln!("cpl-conform: {e}");
            return ExitCode::FAILURE;
        }
        println!("cpl-conform: wrote {}", sb_path.display());
    }

    if exit_fail {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn fuzz(args: &[String]) -> ExitCode {
    let cxx = !args.iter().any(|a| a == "--no-cxx");
    let nums: Vec<u64> = args.iter().filter_map(|a| a.parse().ok()).collect();
    let start = nums.first().copied().unwrap_or(0);
    let count = nums.get(1).copied().unwrap_or(200);
    let keep = find_repo_root()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("target/fuzz");
    let started = Instant::now();
    let findings = cpl_conform::fuzz::run(start, count, cxx, &keep);
    for f in &findings {
        let first = f
            .detail
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("");
        println!("seed {}: {}: {first}", f.seed, f.kind);
    }
    println!(
        "fuzz: {}/{count} generated programs agree ({:.1}s){}",
        count - findings.len() as u64,
        started.elapsed().as_secs_f64(),
        if findings.is_empty() {
            String::new()
        } else {
            format!("; programs in {}", keep.display())
        }
    );
    if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
