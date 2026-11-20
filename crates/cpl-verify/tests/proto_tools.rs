//! The audits and the benchmark against the Python prototype, plus the
//! prototype bugs the fuzzer has found, kept as fixed cases.

use cpl_verify::backend::{repo_root, scratch_dir, Cxx, ProtoTranspiler};
use cpl_verify::diff::{check_source, Expect, FindingKind, Toolchain};
use cpl_verify::{bench, layout, loc};

fn proto() -> Toolchain {
    Toolchain {
        transpiler: Box::new(ProtoTranspiler::new(&repo_root().unwrap())),
        cxx: Cxx::sanitized(),
        interpreter: None,
        traps: vec![],
        static_refusals: vec![],
    }
}

fn assert_prints(source: &str, expected: &str) {
    let finding = check_source(&proto(), source, &Expect::Output(expected.into()));
    assert!(finding.is_none(), "{source}\n{finding:?}");
}

#[test]
fn remainder_alone_compiles() {
    // Found by hand while building the fuzzer: `%` without `/` emitted a
    // call to a runtime helper that was left out of the C++.
    assert_prints("let z: Int = 3\nprint((-5) % z, 5 % (-z))\n", "1 2\n");
}

#[test]
fn literal_arithmetic_is_64_bit() {
    // Seed 28: two literals multiplied as C++ `int` and wrapped.
    assert_prints(
        "print(948332410 * 97, 2147483647 + 9, (-2147483647) - 9)\n",
        "91988243770 2147483656 -2147483656\n",
    );
}

#[test]
fn a_declared_var_keeps_its_declared_range() {
    // Seed 146: the range of a `var x: Int` was inferred from its later
    // assignments alone, so `x / 2` on a negative `x` truncated.
    assert_prints(
        "var x: Int = -41\nlet xs: List<Int> = [x / 2]\nx += 0\nprint(xs)\n",
        "-21\n",
    );
}

#[test]
fn division_and_remainder_match_the_reference() {
    let source = "let a: Int = 7\nlet b: Int = -7\nprint(a / 2, b / 2, a / (-2), b / (-2))\nprint(a % 2, b % 2, a % (-2), b % (-2))\n";
    assert_prints(source, "3 -4 -3 4\n1 1 1 1\n");
}

#[test]
fn an_index_out_of_range_stops_cleanly() {
    let source = "let xs: List<Int> = [1, 2]\nlet i: Int = 2\nprint(xs[0])\nprint(xs[i])\n";
    assert!(check_source(&proto(), source, &Expect::Trap("1\n".into())).is_none());
    // The same check notices a program that does not stop.
    let finding = check_source(&proto(), "print(1)\n", &Expect::Trap("1\n".into())).unwrap();
    assert_eq!(finding.kind, FindingKind::MissedTrap);
}

#[test]
fn undefined_behavior_in_emitted_cxx_is_a_finding() {
    // The prototype does not trap overflow yet, so UBSan sees it. This
    // pins that the harness reports it rather than passing it.
    let source = "let big: Int = 9223372036854775807\nlet one: Int = 1\nprint(big + one)\n";
    let finding = check_source(&proto(), source, &Expect::Trap(String::new())).unwrap();
    assert_eq!(finding.kind, FindingKind::Sanitizer, "{finding:?}");
}

#[test]
fn core_types_have_the_sizes_the_plan_promises() {
    let root = repo_root().unwrap();
    let probe = root.join("crates/cpl-verify/layout/core.cp");
    let work = scratch_dir("layout").unwrap();
    let measured =
        layout::audit(&ProtoTranspiler::new(&root), &Cxx::release(), &probe, &work).unwrap();
    let _ = std::fs::remove_dir_all(&work);
    assert_eq!(measured.len(), 9);
    let problems = layout::problems(&measured, &[]);
    assert!(
        problems.is_empty(),
        "{}\n{}",
        layout::report(&measured),
        problems.join("\n")
    );
}

#[test]
fn every_port_has_both_line_counts() {
    let ports = loc::measure_dir(&repo_root().unwrap().join("proto/ports")).unwrap();
    assert!(ports.len() >= 11, "{} ports", ports.len());
    for port in &ports {
        let (stripped, _) = port
            .stripped
            .as_ref()
            .unwrap_or_else(|| panic!("{} has no .strip file", port.name));
        assert!(
            port.port > 0 && *stripped > 0 && *stripped < port.original,
            "{port:?}"
        );
    }
    let report = loc::report(&ports);
    assert!(
        report.contains("library-stripped: 11 of 11 ports") || ports.len() > 11,
        "{report}"
    );
}

#[test]
fn the_benchmark_times_every_stage_it_has_a_tool_for() {
    let root = repo_root().unwrap();
    let work = scratch_dir("bench").unwrap();
    let mut release = Cxx::release();
    release.build_pch(&work.join("pch")).unwrap();
    let programs = bench::programs(
        &root.join("proto/ports"),
        &root.join("crates/cpl-verify/bench"),
    )
    .unwrap();
    assert!(!programs.is_empty());
    let stub = [
        env!("CARGO_BIN_EXE_cpl-verify").to_string(),
        "lsp-stub".to_string(),
    ];
    let tools = bench::Tools {
        transpiler: &ProtoTranspiler::new(&root),
        release: &release,
        interpreter: None,
        lsp: Some(&stub),
    };
    let report = bench::run(&tools, &programs[..1], 2, &work);
    let _ = std::fs::remove_dir_all(&work);
    let text = report.render();
    for stage in &report.stages {
        match stage.name {
            "debug run" => assert!(
                matches!(stage.outcome, bench::Outcome::NotMeasured(_)),
                "{text}"
            ),
            _ => assert!(
                matches!(&stage.outcome, bench::Outcome::Timed(runs) if runs.len() == 2),
                "{text}"
            ),
        }
    }
    assert_eq!(report.stages.len(), 5, "{text}");
    assert_eq!(report.emitted_bytes.len(), 1, "{text}");
}
