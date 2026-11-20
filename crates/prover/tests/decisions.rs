//! The rows of `decisions.md` the prover answers for: the range of
//! `Int` (1.1), Euclidean division (1.9), laws (3.4), overflow (7.2),
//! contracts as facts (7.9), and `law disjoint` (5.13).

use cpl_prover::{Aliasing, Context, Interval, Law, Lend, Loop, Step, Trouble, Verdict};

fn fails_for(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Refuted(counterexample) => counterexample.to_string(),
        other => panic!("expected a counterexample, got {other:?}"),
    }
}

// ---- 1.1: Int is 64 bits with MIN = -(2^63 - 1) ----------------------------

#[test]
fn int_is_symmetric_around_zero() {
    let mut ctx = Context::new();
    ctx.var("a", "Int").unwrap();
    assert!(ctx
        .prove("Int.MIN <= a and a <= Int.MAX and Int.MIN == -Int.MAX")
        .unwrap()
        .is_proved());
    assert_eq!(
        ctx.range_of("a").unwrap(),
        Interval::new(-i64::MAX as i128, i64::MAX as i128)
    );
    // Symmetric, so negation and abs never leave Int.
    assert!(ctx.check_overflow("-a").unwrap().is_empty());
    assert!(ctx
        .check_overflow("abs(a) - 1")
        .unwrap()
        .iter()
        .all(|f| f.verdict.is_proved()));
    assert_eq!(
        fails_for(&ctx.check_overflow("a - 1").unwrap()[0].verdict),
        "a = Int.MIN"
    );
    assert_eq!(fails_for(&ctx.prove("a > Int.MIN").unwrap()), "a = Int.MIN");
}

// ---- 1.9: Euclidean division and modulo --------------------------------------

#[test]
fn division_is_euclidean_for_every_sign() {
    let mut ctx = Context::new();
    ctx.var("a", "Int").unwrap();
    ctx.var("b", "Int").unwrap();
    ctx.assume("b != 0").unwrap();
    // a == (a / b) * b + a % b with 0 <= a % b < abs(b).
    assert!(ctx.prove("a == (a / b) * b + a % b").unwrap().is_proved());
    assert!(ctx
        .prove("0 <= a % b and a % b < abs(b)")
        .unwrap()
        .is_proved());
    // Rounding is toward minus infinity for a positive divisor, not toward zero.
    assert!(ctx
        .prove("-7 / 2 == -4 and -7 % 2 == 1 and 7 / -2 == -3 and 7 % -2 == 1")
        .unwrap()
        .is_proved());
    assert_eq!(
        fails_for(
            &ctx.prove("a / 2 * 2 <= a and a / 2 * 2 > a - 2 and (-a) / 2 == -(a / 2)")
                .unwrap()
        ),
        "a = 1"
    );
}

#[test]
fn a_proved_sign_selects_the_native_operator() {
    // "When the range checker proves a >= 0 and b > 0, both lower to the native / and %."
    let mut ctx = Context::new();
    ctx.var("a", "0..=1e18").unwrap();
    ctx.var("b", "1..=1e9").unwrap();
    assert!(ctx.prove("a >= 0 and b > 0").unwrap().is_proved());
    assert!(ctx
        .prove("a / b >= 0 and a / b <= a and a % b <= b - 1")
        .unwrap()
        .is_proved());
    // A possibly negative operand needs the fix-up.
    ctx.var("d", "-5..=5").unwrap();
    assert_eq!(fails_for(&ctx.prove("d >= 0").unwrap()), "d = -1");
}

#[test]
fn division_by_zero_is_found_or_ruled_out() {
    let mut ctx = Context::new();
    ctx.var("total", "0..=1e18").unwrap();
    ctx.var("count", "0..=2e5").unwrap();
    let findings = ctx.check_traps("total / count").unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].trouble, Trouble::DivisionByZero);
    assert_eq!(fails_for(&findings[0].verdict), "total = 0, count = 0");
    // A guard in the expression itself is enough.
    assert!(ctx
        .check_traps("if count == 0: 0 else: total / count")
        .unwrap()[0]
        .verdict
        .is_proved());
    assert!(
        ctx.check_traps("count != 0 and total / count > 3").unwrap()[0]
            .verdict
            .is_proved()
    );
    assert!(
        !ctx.check_traps("count != 0 or total / count > 3").unwrap()[0]
            .verdict
            .is_proved()
    );
}

// ---- 7.2: overflow -----------------------------------------------------------

#[test]
fn proven_safe_arithmetic_needs_no_check() {
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("i", "0..n").unwrap();
    ctx.array("a", "0..=1e9").unwrap();
    ctx.array("b", "0..=1e9").unwrap();
    let findings = ctx.check_overflow("a[i] * b[i] + n").unwrap();
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().all(|finding| finding.verdict.is_proved()));
    assert_eq!(findings[0].to_string(), "`a[i] * b[i]` cannot overflow Int");
    assert_eq!(
        findings[0].range,
        Some(Interval::new(0, 1_000_000_000_000_000_000))
    );
}

#[test]
fn a_reachable_overflow_comes_with_its_bound() {
    // error: `total += a[i] * b[i]` can reach 2.0e23 (> 9.2e18)
    //        a: 0..=1e9, b: 0..=1e9, loop over V (len <= 2e5)
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("i", "0..n").unwrap();
    ctx.array("a", "0..=1e9").unwrap();
    ctx.array("b", "0..=1e9").unwrap();
    let finding = ctx.check_sum("a[i] * b[i]", "n").unwrap();
    assert_eq!(
        finding.range,
        Some(Interval::new(0, 200_000_000_000_000_000_000_000))
    );
    let counterexample = finding
        .verdict
        .counterexample()
        .expect("the sum can overflow");
    let (count, step) = (
        counterexample.int("n").unwrap(),
        counterexample.int("a[i] * b[i]").unwrap(),
    );
    assert!(
        count * step > i64::MAX as i128 && count <= 200_000 && step <= 1_000_000_000_000_000_000
    );
    assert!(finding
        .to_string()
        .starts_with("`the sum of n values of a[i] * b[i]` can overflow Int: n = "));
    // help: tighten a bound with `assert`.
    ctx.assume("a[i] <= 1e6 and b[i] <= 1e6").unwrap();
    let tightened = ctx.check_sum("a[i] * b[i]", "n").unwrap();
    assert!(tightened.verdict.is_proved(), "{tightened}");
    assert_eq!(
        tightened.range,
        Some(Interval::new(0, 200_000_000_000_000_000))
    );
}

#[test]
fn an_unbounded_value_is_neither_safe_nor_a_proven_overflow_of_the_program() {
    // Unknown inputs: the prover shows values that overflow, which is what a
    // local build would trap on; a judge build runs the plain operator.
    let mut ctx = Context::new();
    ctx.var("x", "Int").unwrap();
    ctx.var("y", "Int").unwrap();
    let findings = ctx.check_overflow("x * y + 1").unwrap();
    assert_eq!(findings.len(), 2);
    let product = findings[0]
        .verdict
        .counterexample()
        .expect("x * y can overflow");
    let (x, y) = (product.int("x").unwrap(), product.int("y").unwrap());
    assert!(x.checked_mul(y).is_none_or(|p| p.abs() > i64::MAX as i128));
    let max = i64::MAX as i128;
    assert_eq!(
        findings[0].range,
        Some(Interval::new(-max * max, max * max))
    );
}

#[test]
fn overflow_inside_a_guard_is_checked_under_the_guard() {
    let mut ctx = Context::new();
    ctx.var("x", "Int").unwrap();
    assert!(ctx.check_overflow("if x < 1e18: x + 1 else: x").unwrap()[0]
        .verdict
        .is_proved());
    assert!(
        !ctx.check_overflow("if x > 1e18: x + 1 else: x").unwrap()[0]
            .verdict
            .is_proved()
    );
    // The midpoint that overflows and the one that does not.
    ctx.var("lo", "0..=Int.MAX").unwrap();
    ctx.var("hi", "0..=Int.MAX").unwrap();
    ctx.assume("lo <= hi").unwrap();
    assert_eq!(
        fails_for(&ctx.check_overflow("(lo + hi) / 2").unwrap()[0].verdict),
        "lo = 1, hi = Int.MAX"
    );
    assert!(ctx
        .check_overflow("lo + (hi - lo) / 2")
        .unwrap()
        .iter()
        .all(|f| f.verdict.is_proved()));
    assert!(ctx
        .prove("lo + (hi - lo) / 2 in lo..=hi")
        .unwrap()
        .is_proved());
}

#[test]
fn a_goal_is_not_refuted_by_a_run_that_overflows() {
    // `a + 1 > a` holds wherever `a + 1` is an Int; the overflow is its own finding.
    let mut ctx = Context::new();
    ctx.var("a", "Int").unwrap();
    assert!(ctx.prove("a + 1 > a").unwrap().is_proved());
    assert_eq!(
        fails_for(&ctx.check_overflow("a + 1").unwrap()[0].verdict),
        "a = Int.MAX"
    );
}

// ---- 7.9: contracts are facts ---------------------------------------------------

#[test]
fn requires_and_ensures_feed_the_proof() {
    // fn query(st, s: Range1<V>) -> Int  requires s.lo <= s.hi  ensures result >= 0
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("s", "(lo: 0..n, hi: 0..n)").unwrap();
    assert_eq!(
        fails_for(&ctx.prove("s.hi - s.lo + 1 >= 1").unwrap()),
        "n = 2, s = (lo: 1, hi: 0)"
    );
    ctx.assume("s.lo <= s.hi").unwrap();
    assert!(ctx.prove("s.hi - s.lo + 1 in 1..=n").unwrap().is_proved());
    ctx.function("query", "Int").unwrap();
    ctx.assume("query(s.lo, s.hi) >= 0").unwrap();
    assert!(ctx.check_overflow("query(s.lo, s.hi) - 1").unwrap()[0]
        .verdict
        .is_proved());
}

#[test]
fn the_two_pointer_invariant() {
    // var l = V.first
    // for r in V:
    //     while a[r] - a[l] > k: l = l.next ?? break
    //     invariant l <= r
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("k", "0..=1e9").unwrap();
    ctx.var("r", "0..n").unwrap();
    ctx.var("l", "0..n").unwrap();
    ctx.array("a", "-1e9..=1e9").unwrap();
    ctx.assume("l <= r").unwrap();
    let inner = Loop::new("a[r] - a[l] > k")
        .unwrap()
        .invariant("l <= r")
        .unwrap()
        .step(Step::assign("l", "l + 1").unwrap());
    let report = ctx.check_loop(&inner).unwrap();
    assert!(report.holds(), "{report}");
    // The invariant is what makes the window length a valid count.
    let after = ctx.after_loop(&inner).unwrap();
    assert!(after
        .clone()
        .prove("r - l + 1 in 1..=n and a[r] - a[l] <= k")
        .unwrap()
        .is_proved());
}

#[test]
fn an_invariant_that_the_body_breaks() {
    // With a negative k the window can run past r: a[r] - a[r] = 0 > k.
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("k", "-1e9..=1e9").unwrap();
    ctx.var("r", "0..n").unwrap();
    ctx.var("l", "0..n").unwrap();
    ctx.array("a", "-1e9..=1e9").unwrap();
    ctx.assume("l <= r").unwrap();
    let inner = Loop::new("a[r] - a[l] > k")
        .unwrap()
        .invariant("l <= r")
        .unwrap()
        .step(Step::assign("l", "l + 1").unwrap());
    let report = ctx.check_loop(&inner).unwrap();
    assert!(report.entry[0].1.is_proved());
    assert_eq!(
        report.to_string(),
        "invariant l <= r on entry: proved\n\
         invariant l <= r after the body: fails for n = 1, k = -1, r = 0, l = 1"
    );
}

#[test]
fn an_invariant_that_does_not_hold_on_entry() {
    let mut ctx = Context::new();
    ctx.var("n", "0..=2e5").unwrap();
    ctx.var("i", "Int").unwrap();
    ctx.var("total", "Int").unwrap();
    ctx.assume("i == 0 and total == 1").unwrap();
    // total == 2 * i fails at the start; total == 2 * i + 1 is kept by the body.
    let body = [
        Step::assign("total", "total + 2").unwrap(),
        Step::assign("i", "i + 1").unwrap(),
    ];
    let wrong = Loop {
        guard: cpl_prover::parse::term("i < n").unwrap(),
        invariants: vec![],
        body: body.to_vec(),
    };
    let wrong = wrong
        .invariant("total == 2 * i")
        .unwrap()
        .invariant("0 <= i and i <= n")
        .unwrap();
    let report = ctx.check_loop(&wrong).unwrap();
    assert_eq!(fails_for(&report.entry[0].1), "i = 0, total = 1");
    assert!(report
        .preserved
        .iter()
        .all(|(_, verdict)| verdict.is_proved()));
    assert!(report.entry[1].1.is_proved());
    let right = Loop {
        invariants: vec![],
        ..wrong
    }
    .invariant("total == 2 * i + 1")
    .unwrap()
    .invariant("0 <= i and i <= n")
    .unwrap();
    assert!(ctx.check_loop(&right).unwrap().holds());
    // After the loop the invariant and the failed guard give the result.
    assert!(ctx
        .after_loop(&right)
        .unwrap()
        .prove("total == 2 * n + 1")
        .unwrap()
        .is_proved());
}

#[test]
fn a_branch_in_the_body_is_checked_on_both_paths() {
    // while lo < hi: mid = lo + (hi - lo) / 2; if ok(mid): hi = mid else: lo = mid + 1
    let mut ctx = Context::new();
    ctx.var("n", "1..=1e9").unwrap();
    ctx.var("lo", "Int").unwrap();
    ctx.var("hi", "Int").unwrap();
    ctx.var("mid", "Int").unwrap();
    ctx.function("ok", "Bool").unwrap();
    ctx.assume("lo == 0 and hi == n").unwrap();
    let search = Loop::new("lo < hi")
        .unwrap()
        .invariant("0 <= lo and lo <= hi and hi <= n")
        .unwrap()
        .step(Step::assign("mid", "lo + (hi - lo) / 2").unwrap())
        .step(
            Step::branch(
                "ok(mid)",
                vec![Step::assign("hi", "mid").unwrap()],
                vec![Step::assign("lo", "mid + 1").unwrap()],
            )
            .unwrap(),
        );
    assert_eq!(search.modified(), ["mid", "hi", "lo"]);
    let report = ctx.check_loop(&search).unwrap();
    assert!(report.holds(), "{report}");
    assert!(ctx
        .after_loop(&search)
        .unwrap()
        .prove("lo == hi")
        .unwrap()
        .is_proved());
    // `lo = mid` instead of `mid + 1` keeps the invariant but the range stops shrinking:
    // the variant `hi - lo` no longer decreases.
    let mut at_top = ctx.at_iteration(&search).unwrap();
    at_top.assign("before", "hi - lo").unwrap();
    at_top.assign("mid", "lo + (hi - lo) / 2").unwrap();
    let mut stuck = at_top.clone();
    stuck.assign("lo", "mid").unwrap();
    assert_eq!(
        fails_for(&stuck.prove("hi - lo < before").unwrap()),
        "n = 1, hi = 1, before = 1, mid = 0, lo = 0"
    );
    at_top.assign("lo", "mid + 1").unwrap();
    assert!(at_top.prove("hi - lo < before").unwrap().is_proved());
}

// ---- 5.13 and 5.5: law disjoint ----------------------------------------------------

#[test]
fn every_kind_of_flow_fact_proves_the_law() {
    let law = Law::parse("law disjoint(i, j) = i != j").unwrap();
    let check = |ctx: &mut Context| {
        ctx.check_lends(
            &Lend::parse("mut a[i]").unwrap(),
            &Lend::parse("mut a[j]").unwrap(),
            Some(&law),
        )
        .unwrap()
    };
    let base = || {
        let mut ctx = Context::new();
        ctx.var("n", "2..=2e5").unwrap();
        ctx.var("i", "0..n").unwrap();
        ctx.var("j", "0..n").unwrap();
        ctx
    };
    assert!(!check(&mut base()).is_allowed());
    // `if`, `while`, `assert`, `requires`: a condition that dominates the lends.
    for fact in [
        "i != j",
        "i < j",
        "j + 1 <= i",
        "not (i == j)",
        "i == 0 and j == n - 1",
        "j == i + 1",
        "i % 2 != j % 2",
    ] {
        let mut ctx = base();
        ctx.assume(fact).unwrap();
        assert_eq!(check(&mut ctx), Aliasing::Disjoint, "{fact}");
    }
    // Value ranges: the two halves of a domain never meet.
    let mut ctx = Context::new();
    ctx.var("n", "2..=2e5").unwrap();
    ctx.var("i", "0..n / 2").unwrap();
    ctx.var("j", "n / 2..n").unwrap();
    assert_eq!(check(&mut ctx), Aliasing::Disjoint);
    // Distinct literals.
    let literals = (
        Lend::parse("mut a[0]").unwrap(),
        Lend::parse("mut a[1]").unwrap(),
    );
    assert_eq!(
        base()
            .check_lends(&literals.0, &literals.1, Some(&law))
            .unwrap(),
        Aliasing::Disjoint
    );
    // A fact that only makes it likely is not a proof.
    for fact in ["i <= j", "i != j or n > 5", "i + j != 3"] {
        let mut ctx = base();
        ctx.assume(fact).unwrap();
        assert!(!check(&mut ctx).is_allowed(), "{fact}");
    }
}

#[test]
fn swap_and_take_are_library_functions_over_at() {
    // pub fn swap(mut self, i: I, j: I): if i != j: swap(mut self[i], mut self[j])
    let law = Law::parse("law disjoint(i, j) = i != j").unwrap();
    let mut ctx = Context::new();
    ctx.var("K", "1..=2e5").unwrap();
    ctx.var("i", "0..K").unwrap();
    ctx.var("j", "0..K").unwrap();
    let (a, b) = (
        Lend::parse("mut self[i]").unwrap(),
        Lend::parse("mut self[j]").unwrap(),
    );
    match ctx.check_lends(&a, &b, Some(&law)).unwrap() {
        Aliasing::Conflict {
            reason,
            counterexample,
        } => {
            assert_eq!(reason, "`i != j` does not hold");
            assert_eq!(counterexample.unwrap().to_string(), "K = 1, i = 0, j = 0");
        }
        other => panic!("{other:?}"),
    }
    ctx.assume("i != j").unwrap();
    assert_eq!(
        ctx.check_lends(&a, &b, Some(&law)).unwrap(),
        Aliasing::Disjoint
    );
}
