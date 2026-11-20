//! The prover runs inside the front end, which has 50 ms for a whole
//! file. These bounds are loose enough for an unoptimised build on a
//! busy machine; `cargo run -p cpl-prover --example tour --release`
//! prints the real numbers, which are microseconds.

use std::time::{Duration, Instant};

use cpl_prover::{Context, Law, Lend, Structure};

/// The fastest of three runs, so one scheduling hiccup doesn't fail the test.
fn best_of_three(mut run: impl FnMut()) -> Duration {
    (0..3)
        .map(|_| {
            let start = Instant::now();
            run();
            start.elapsed()
        })
        .min()
        .unwrap()
}

#[test]
fn the_checks_of_a_typical_function_fit_the_budget() {
    // What one function body asks: a handful of bounds, a pair of lends,
    // a few overflow checks.
    let law = Law::parse("law disjoint(i, j) = i != j").unwrap();
    let elapsed = best_of_three(|| {
        let mut ctx = Context::new();
        ctx.var("n", "1..=2e5").unwrap();
        ctx.var("i", "0..n").unwrap();
        ctx.var("j", "0..n").unwrap();
        ctx.array("a", "0..=1e9").unwrap();
        ctx.assume("i < j").unwrap();
        for index in ["i", "j", "i + 1", "j - 1", "(i + j) / 2", "n - 1 - i"] {
            assert!(ctx.check_index(index, "0..n").unwrap().is_proved());
        }
        let (x, y) = (
            Lend::parse("mut a[i]").unwrap(),
            Lend::parse("mut a[j]").unwrap(),
        );
        assert!(ctx.check_lends(&x, &y, Some(&law)).unwrap().is_allowed());
        for expr in [
            "a[i] + a[j]",
            "a[i] * a[j]",
            "a[i] * n",
            "(a[j] - a[i]) * (j - i)",
        ] {
            assert!(ctx
                .check_overflow(expr)
                .unwrap()
                .iter()
                .all(|finding| finding.verdict.is_proved()));
        }
        assert!(ctx.prove("j + 1 < n").unwrap().counterexample().is_some());
    });
    assert!(elapsed < Duration::from_millis(50), "took {elapsed:?}");
}

#[test]
fn a_segment_tree_monoid_is_checked_within_the_budget() {
    let elapsed = best_of_three(|| {
        let op = "(x, y) => (sum: x.sum + y.sum, pre: max(x.pre, x.sum + y.pre), \
                  suf: max(y.suf, y.sum + x.suf), best: max(x.best, y.best, x.suf + y.pre))";
        let report = Structure::semigroup().check(
            &[("T", "(sum: Int, pre: Int, suf: Int, best: Int)")],
            &[("op", op)],
        );
        assert!(report.unwrap().holds());
        let count = [
            ("id", "(Int.MIN, 0)"),
            (
                "op",
                "(x, y) => if x.0 > y.0: x else: if y.0 > x.0: y else: (x.0, x.1 + y.1)",
            ),
        ];
        assert!(Structure::monoid()
            .check(&[("T", "(Int, Int)")], &count)
            .unwrap()
            .holds());
    });
    assert!(elapsed < Duration::from_millis(500), "took {elapsed:?}");
}
