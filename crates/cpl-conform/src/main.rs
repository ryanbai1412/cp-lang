use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use cpl_conform::conformance;
use cpl_conform::scoreboard::{self, Scoreboard, SuiteReport};
use cpl_conform::{Suite, default_scoreboard_path, find_repo_root, run_suites};

const USAGE: &str = "\
usage: cpl-conform [conformance|corpus|docs|diff|all]... [options] [filter]

suites:
  conformance     spec/conformance/**/*.cp cases
  corpus          corpus/problems/<id>/spec.cp on tests/*.in vs *.ans
  docs            /// example: lines in spec/std/*.cp
  diff            interpreter vs g++ -O2 -std=c++17 differential
  all             all four suites (default if no suite is named)

options:
  --check         fail if any suite's pass count drops or a passing case regresses
  --write         update the scoreboard JSON with the suites that were run
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
        println!(
            "{name:12} {:>4}/{:<4} pass ({})",
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
        let ratchet = scoreboard::check_ratchet(&baseline, &current, filter.is_some());
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
