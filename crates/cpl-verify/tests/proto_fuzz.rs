//! The fuzzer against the Python prototype: `proto/cpc/` to C++17, built
//! with `g++ -std=c++17 -O2 -DLOCAL -fsanitize=address,undefined`.

use cpl_verify::backend::{repo_root, scratch_dir, Cxx, ProtoTranspiler};
use cpl_verify::diff::{self, Campaign, Toolchain};
use cpl_verify::fuzz::eval::Trap;

fn proto() -> Toolchain {
    let root = repo_root().expect("the repository root");
    let mut cxx = Cxx::sanitized();
    cxx.build_pch(&scratch_dir("pch").unwrap())
        .expect("precompiling <bits/stdc++.h>");
    Toolchain {
        transpiler: Box::new(ProtoTranspiler::new(&root)),
        cxx,
        interpreter: None,
        // A local build of the prototype stops on all three.
        traps: vec![Trap::Overflow, Trap::DivByZero, Trap::IndexOutOfRange],
        static_refusals: vec!["beyond Int".into()],
    }
}

/// `CPL_FUZZ_SEEDS=0..2000 cargo test -p cpl-verify --test proto_fuzz`
/// runs a longer campaign.
fn seeds() -> std::ops::Range<u64> {
    match std::env::var("CPL_FUZZ_SEEDS") {
        Ok(s) => {
            let (a, b) = s.split_once("..").expect("CPL_FUZZ_SEEDS is `start..end`");
            a.parse().unwrap()..b.parse().unwrap()
        }
        Err(_) => 0..40,
    }
}

#[test]
fn prototype_agrees_with_the_reference_evaluator() {
    let report = diff::run(&proto(), &Campaign::new(seeds()));
    let shown: Vec<String> = report
        .failures
        .iter()
        .map(|f| {
            format!(
                "seed {}: {:?}: {}\n{}",
                f.seed, f.finding.kind, f.finding.detail, f.shrunk
            )
        })
        .collect();
    assert!(shown.is_empty(), "{}", shown.join("\n"));
    // Refusals are allowed, but not as a way to pass by testing nothing.
    let total = seeds().count() as u64;
    assert!(
        report.refused * 10 <= total,
        "{} of {total} programs refused",
        report.refused
    );
    assert_eq!(report.programs + report.refused, total);
    // Every program also ran once per trap kind, ending in that error.
    assert_eq!(report.trap_programs, 3 * report.programs);
}
