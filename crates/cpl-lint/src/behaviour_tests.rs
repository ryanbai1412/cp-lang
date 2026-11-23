//! Fixes that once changed what a program does; `cpl-cli`'s
//! `fix_behavior` test runs every runnable program before and after them.

use super::*;
use crate::tests::{all_on, fixed};

#[test]
fn a_var_a_closure_changes_is_changed() {
    // spec/conformance/core-05/closure-captures-at-call.cp: `() => c += 1`
    // has no parameters, and its body was skipped as if it were `.cost`.
    let src = "var c = 0\nlet inc = () => c += 1\ninc()\nprint(c)\n";
    assert!(lint(src, &all_on())
        .iter()
        .all(|d| d.code != VAR_NEVER_REASSIGNED.code));
}

#[test]
fn loop_reduction_fixes_run() {
    // core-01/range-forms.cp: `(0..10 by 3).sum()` doesn't run yet (E0501);
    // tests/programs/repeat_loop.cp: `repeat n:` became `sum(1 for repeat in ..)`.
    let src =
        "var s = 0\nfor i in 0..10 by 3: s += i\nprint(s)\nvar t = 0\nrepeat 4: t += 1\nprint(t)\n";
    assert_eq!(
        fixed(src),
        "let s = sum(i for i in 0..10 by 3)\nprint(s)\nvar t = 0\nrepeat 4: t += 1\nprint(t)\n"
    );
}

#[test]
fn fixes_that_changed_what_programs_do() {
    // spec/conformance/core-01/shared-annotation.cp: `a` and `b` share one `var`.
    let shared = "var a, b: Int = 4\na += 1\nprint(a, b)\n";
    assert!(lint(shared, &all_on())
        .iter()
        .all(|d| d.code != VAR_NEVER_REASSIGNED.code));
    // tests/programs/dream_gaps.cp: `let sum = sum(..)` names itself.
    let named = "let n = 3\nvar sum = 0\nfor i in 0..n:\n    sum += i * 2\nprint(sum)\n";
    assert!(lint(named, &all_on())
        .iter()
        .all(|d| d.code != LOOP_REDUCTION.code));
    // tests/programs/reflect_fields.cp: `fields(..)` is unrolled, not a sequence.
    let reflect = "struct E(u: Int, v: Int)\nfn n(a: E) -> Int:\n    var n = 0\n    for _ in fields(a):\n        n += 1\n    return n\nprint(n(E(u: 1, v: 2)))\n";
    assert!(lint(reflect, &all_on())
        .iter()
        .all(|d| d.code != LOOP_REDUCTION.code));
    // std-tree/rooted-single-preorder.cp: `t.preorder(v)` walks the subtree of `v`.
    let walk = "domain V = 0..1\nlet t = Rooted([[]] as [V]List<V>, root: V(0))\nprint(t.preorder(V(0)).count())\n";
    assert!(lint(walk, &all_on())
        .iter()
        .all(|d| d.code != TREE_ORDER.code));
}

#[test]
fn a_keep_where_a_value_is_wanted_changes_its_place() {
    // crates/cpl-cli/tests/programs/s2_keep_bool.cp: `let a = best min= x`
    // writes `best`, so `var best` stays a `var`.
    let src = "var best = 9\nlet a = best min= 4\nvar hi = 0\nfor x in [3, 1]:\n    if hi max= x: print(x)\nprint(best, a, hi)\n";
    let diags = crate::lint(src, &crate::Config::default());
    assert!(diags.iter().all(|d| d.code.0 != "L0201"), "{diags:?}");
}

#[test]
fn an_index_loop_that_stores_its_item_keeps_the_index() {
    let src = "let names = [\"b\", \"a\"]\nvar first = names[0]\nfor i in 0..names.len:\n    if names[i] < first: first = names[i]\nprint(first)\n";
    let diags = crate::lint(src, &crate::Config::default());
    assert!(
        diags.iter().all(|d| d.code != crate::INDEX_ONLY_LOOP.code),
        "{diags:?}"
    );
}
