//! The cases of `plan/19-index-and-lends.md`: bounded indices, slices,
//! grids, and the `law disjoint` rules of sections 3.6, 5.4 and 5.6.

use cpl_prover::{Aliasing, Context, Law, Lend, Trouble, Verdict};

/// `fn at(self, i: I) lends mut T: law disjoint(i, j) = i != j`.
fn element_law() -> Law {
    Law::parse("law disjoint(i, j) = i != j").unwrap()
}

/// `fn at(self, s: Range<I>) lends mut Slice<I, T>`.
fn slice_law() -> Law {
    Law::parse("law disjoint(s, t) = s.hi <= t.lo or t.hi <= s.lo").unwrap()
}

/// A graph-sized domain `V = 0..n` with two positions in it.
fn two_positions() -> Context {
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("i", "0..n").unwrap();
    ctx.var("j", "0..n").unwrap();
    ctx
}

fn lends(ctx: &mut Context, a: &str, b: &str, law: &Law) -> Aliasing {
    ctx.check_lends(
        &Lend::parse(a).unwrap(),
        &Lend::parse(b).unwrap(),
        Some(law),
    )
    .unwrap()
}

fn conflict_at(aliasing: &Aliasing) -> String {
    match aliasing {
        Aliasing::Conflict {
            counterexample: Some(counterexample),
            ..
        } => counterexample.to_string(),
        other => panic!("expected a conflict with a counterexample, got {other:?}"),
    }
}

fn fails_for(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Refuted(counterexample) => counterexample.to_string(),
        other => panic!("expected a counterexample, got {other:?}"),
    }
}

// ---- 3.6 and 5.4: two lends of one array -------------------------------

#[test]
fn two_elements_need_a_fact() {
    // merge_into(mut cnt[i], cnt[j]): cnt[i] and cnt[j] may be the same element.
    let mut ctx = two_positions();
    let unguarded = lends(&mut ctx, "mut cnt[i]", "cnt[j]", &element_law());
    assert_eq!(conflict_at(&unguarded), "n = 1, i = 0, j = 0");
    match &unguarded {
        Aliasing::Conflict { reason, .. } => assert_eq!(reason, "`i != j` does not hold"),
        _ => unreachable!(),
    }
    // if i != j: merge_into(mut cnt[i], cnt[j])
    ctx.assume("i != j").unwrap();
    assert_eq!(
        lends(&mut ctx, "mut cnt[i]", "cnt[j]", &element_law()),
        Aliasing::Disjoint
    );
}

#[test]
fn the_same_element_twice_is_rejected() {
    // 5.6 case 2: swap(mut a[i], mut a[i]).
    let mut ctx = two_positions();
    let twice = lends(&mut ctx, "mut a[i]", "mut a[i]", &element_law());
    assert_eq!(conflict_at(&twice), "n = 1, i = 0");
    assert!(!twice.is_allowed());
}

#[test]
fn reads_never_conflict() {
    let mut ctx = two_positions();
    assert_eq!(
        lends(&mut ctx, "a[i]", "a[j]", &element_law()),
        Aliasing::Shared
    );
    assert_eq!(
        lends(&mut ctx, "a[i]", "a[i]", &element_law()),
        Aliasing::Shared
    );
}

#[test]
fn a_continue_leaves_the_fact_behind() {
    // for i in V: for j in V: if i == j: continue; absorb(mut bag[i], bag[j])
    let mut ctx = two_positions();
    ctx.assume("not (i == j)").unwrap();
    assert_eq!(
        lends(&mut ctx, "mut bag[i]", "bag[j]", &element_law()),
        Aliasing::Disjoint
    );
}

#[test]
fn an_ordered_pair_is_a_pair_of_different_positions() {
    // for (u, v) in pairs(V): the refinement u < v gives u != v.
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("u", "0..n").unwrap();
    ctx.var("v", "0..n").unwrap();
    ctx.assume("u < v").unwrap();
    assert_eq!(
        lends(&mut ctx, "mut bag[v]", "bag[u]", &element_law()),
        Aliasing::Disjoint
    );
}

#[test]
fn a_fact_ends_when_its_variable_is_assigned() {
    // var j = i.next!     Seq.next ensures its result is > self
    let mut ctx = two_positions();
    ctx.assign("j", "i + 1").unwrap();
    assert_eq!(
        lends(&mut ctx, "mut bag[i]", "bag[j]", &element_law()),
        Aliasing::Disjoint
    );
    // j = pick()          the fact about j ended at the assignment
    ctx.havoc("j").unwrap();
    let after = lends(&mut ctx, "mut bag[i]", "bag[j]", &element_law());
    assert_eq!(conflict_at(&after), "n = 1, i = 0, j = 0");
}

#[test]
fn assigning_one_index_to_the_other_ends_the_fact() {
    // 5.6 case 3: if i != j: swap(mut a[i], mut a[j]); then the same call after i = j.
    let mut ctx = two_positions();
    ctx.assume("i != j").unwrap();
    assert_eq!(
        lends(&mut ctx, "mut a[i]", "mut a[j]", &element_law()),
        Aliasing::Disjoint
    );
    ctx.assign("i", "j").unwrap();
    let after = lends(&mut ctx, "mut a[i]", "mut a[j]", &element_law());
    assert!(matches!(
        after,
        Aliasing::Conflict {
            counterexample: Some(_),
            ..
        }
    ));
}

#[test]
fn a_call_that_writes_the_index_drops_the_fact() {
    // 5.6 case 14: if i != r: step(); mv(mut a[i], mut a[r]) where step assigns i.
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("i", "0..n").unwrap();
    ctx.var("r", "0..n").unwrap();
    ctx.assume("i != r").unwrap();
    assert!(lends(&mut ctx, "mut a[i]", "mut a[r]", &element_law()).is_allowed());
    ctx.havoc("i").unwrap();
    assert_eq!(
        conflict_at(&lends(&mut ctx, "mut a[i]", "mut a[r]", &element_law())),
        "n = 2, r = 0, i = 0"
    );
}

#[test]
fn different_indices_do_not_make_different_images() {
    // absorb(mut bag[p[i]], bag[p[j]]): error even when i != j, nothing says p[i] != p[j].
    let mut ctx = two_positions();
    ctx.array("p", "0..n").unwrap();
    ctx.assume("i != j").unwrap();
    let images = lends(&mut ctx, "mut bag[p[i]]", "bag[p[j]]", &element_law());
    assert_eq!(
        conflict_at(&images),
        "n = 2, i = 1, j = 0, p[1] = 0, p[0] = 0"
    );
    // A permutation states it, and then the lends are fine.
    ctx.assume("p[i] != p[j]").unwrap();
    assert_eq!(
        lends(&mut ctx, "mut bag[p[i]]", "bag[p[j]]", &element_law()),
        Aliasing::Disjoint
    );
}

#[test]
fn halves_that_share_an_endpoint_are_disjoint() {
    // solve(mut a[l..m], mut a[m..r]): the first range ends where the second starts.
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("l", "0..=n").unwrap();
    ctx.var("m", "0..=n").unwrap();
    ctx.var("r", "0..=n").unwrap();
    assert_eq!(
        lends(&mut ctx, "mut a[l..m]", "mut a[m..r]", &slice_law()),
        Aliasing::Disjoint
    );
    // An inclusive end overlaps the next half by one element.
    let overlap = lends(&mut ctx, "mut a[l..=m]", "mut a[m..r]", &slice_law());
    assert_eq!(conflict_at(&overlap), "n = 1, l = 0, m = 0, r = 1");
    // Ranges known to be apart need no shared endpoint.
    ctx.var("s", "0..=n").unwrap();
    ctx.assume("m < s").unwrap();
    assert_eq!(
        lends(&mut ctx, "mut a[l..m]", "a[s..r]", &slice_law()),
        Aliasing::Disjoint
    );
}

#[test]
fn an_element_and_a_slice_are_different_projections() {
    let mut ctx = two_positions();
    ctx.assume("i < j").unwrap();
    let mixed = lends(&mut ctx, "mut a[i]", "mut a[j..n]", &element_law());
    match mixed {
        Aliasing::Conflict { reason, counterexample: None } => assert_eq!(
            reason,
            "`at` and `slice` are different projections of `a`: a law only relates two uses of one projection"
        ),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_row_and_a_column_always_conflict() {
    // absorb(mut g.row(r), g.col(c)), and 5.6 case 6 with r1 != r2 known.
    let mut ctx = Context::new();
    ctx.var("r1", "0..10").unwrap();
    ctx.var("r2", "0..10").unwrap();
    ctx.assume("r1 != r2").unwrap();
    let row_law = Law::parse("law disjoint(r, q) = r != q").unwrap();
    assert!(!lends(&mut ctx, "mut g.row(r1)", "g.col(r2)", &row_law).is_allowed());
    assert!(!lends(&mut ctx, "g.row(r1)", "mut g[r2, 0]", &row_law).is_allowed());
    // Two rows are one projection, and the law decides.
    assert_eq!(
        lends(&mut ctx, "mut g.row(r1)", "mut g.row(r2)", &row_law),
        Aliasing::Disjoint
    );
}

#[test]
fn different_fields_are_separate_without_a_law() {
    // absorb(mut s.left[i], s.right[i])
    let mut ctx = two_positions();
    let (left, right) = (
        Lend::parse("mut s.left[i]").unwrap(),
        Lend::parse("s.right[i]").unwrap(),
    );
    assert_eq!(
        ctx.check_lends(&left, &right, None).unwrap(),
        Aliasing::Separate
    );
    // A place and a part of it are not separate.
    let (whole, part) = (
        Lend::parse("mut s").unwrap(),
        Lend::parse("s.left[i]").unwrap(),
    );
    assert!(!ctx.check_lends(&whole, &part, None).unwrap().is_allowed());
    // A projection without a law never lends twice.
    let (a, b) = (
        Lend::parse("mut h.top(i)").unwrap(),
        Lend::parse("mut h.top(j)").unwrap(),
    );
    match ctx.check_lends(&a, &b, None).unwrap() {
        Aliasing::Conflict { reason, .. } => assert_eq!(reason, "`top` declares no `law disjoint`"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn pair_mut_and_swap_are_library_functions() {
    // pub fn pair_mut(mut self, i: I, j: I): assert i != j; lend (mut self[i], mut self[j])
    let mut ctx = two_positions();
    ctx.assume("i != j").unwrap();
    assert!(lends(&mut ctx, "mut self[i]", "mut self[j]", &element_law()).is_allowed());
}

// ---- 5.6 case 10: a law that is wrong ------------------------------------

#[test]
fn the_library_laws_are_sound() {
    let ctx = Context::new();
    let element = Law::parse("covers(i, k) = k == i").unwrap();
    assert!(ctx
        .check_disjoint_law(&element_law(), &element)
        .unwrap()
        .is_proved());
    let slice = Law::parse(
        "law disjoint(s: (lo: Int, hi: Int), t: (lo: Int, hi: Int)) = s.hi <= t.lo or t.hi <= s.lo",
    );
    let covers = Law::parse("covers(s, k) = s.lo <= k and k < s.hi").unwrap();
    assert!(ctx
        .check_disjoint_law(&slice.unwrap(), &covers)
        .unwrap()
        .is_proved());
}

#[test]
fn a_wrong_law_is_refuted_with_the_shared_cell() {
    let ctx = Context::new();
    // Off by one: ranges that share their last and first element.
    let loose = Law::parse("law disjoint(s: (lo: Int, hi: Int), t: (lo: Int, hi: Int)) = s.hi <= t.lo + 1 or t.hi <= s.lo");
    let covers = Law::parse("covers(s, k) = s.lo <= k and k < s.hi").unwrap();
    let verdict = ctx.check_disjoint_law(&loose.unwrap(), &covers).unwrap();
    assert_eq!(
        fails_for(&verdict),
        "s = (lo: 0, hi: 1), t = (lo: 0, hi: 1), k = 0"
    );

    // A row law over blocks that overlap: row r lends cells r*W .. r*W + W + 1.
    let mut grid = Context::new();
    grid.var("W", "1..=1000").unwrap();
    let rows = Law::parse("law disjoint(r: 0..1000, q: 0..1000) = r != q").unwrap();
    let block = Law::parse("covers(r, k) = r * W <= k and k < r * W + W + 1").unwrap();
    let verdict = grid.check_disjoint_law(&rows, &block).unwrap();
    assert_eq!(fails_for(&verdict), "W = 1, r = 1, q = 0, k = 1");
    // The honest block is fine.
    let block = Law::parse("covers(r, k) = r * W <= k and k < r * W + W").unwrap();
    assert!(grid.check_disjoint_law(&rows, &block).unwrap().is_proved());
}

// ---- 3.1 and 4.1: bounded indices ----------------------------------------

#[test]
fn a_position_of_a_domain_indexes_its_array() {
    // struct Index over 0..K; Array keeps `invariant buf.cap == I.K`.
    let mut ctx = Context::new();
    ctx.var("K", "0..=2e5").unwrap();
    ctx.var("cap", "Int").unwrap();
    ctx.assume("cap == K").unwrap();
    ctx.var("idx", "0..K").unwrap();
    assert!(ctx.check_index("idx", "0..cap").unwrap().is_proved());
    // An Int that was never checked is not a position.
    ctx.var("raw", "Int").unwrap();
    assert_eq!(
        fails_for(&ctx.check_index("raw", "0..cap").unwrap()),
        "K = 1, cap = 1, raw = 1"
    );
    // a.raw[i] behind a guard is fine.
    ctx.assume("0 <= raw and raw < K").unwrap();
    assert!(ctx.check_index("raw", "0..cap").unwrap().is_proved());
}

#[test]
fn stepping_past_the_end_is_caught() {
    // 5.6 case 22: V.first + k past the end, and the clamped windows z.l(k), z.r(k).
    let mut ctx = Context::new();
    ctx.var("first", "Int").unwrap();
    ctx.var("last", "Int").unwrap();
    ctx.var("z", "first..=last").unwrap();
    ctx.var("k", "0..=1e9").unwrap();
    assert_eq!(
        fails_for(&ctx.check_index("first + k", "first..=last").unwrap()),
        "first = 0, last = 0, k = 1"
    );
    assert!(ctx
        .check_index("max(first, z - k)", "first..=last")
        .unwrap()
        .is_proved());
    assert!(ctx
        .check_index("min(last, z + k)", "first..=last")
        .unwrap()
        .is_proved());
    assert!(ctx
        .prove("max(first, z - k) <= z and z <= min(last, z + k)")
        .unwrap()
        .is_proved());
    // z.next! needs z != last.
    assert_eq!(
        fails_for(&ctx.check_index("z + 1", "first..=last").unwrap()),
        "first = 0, last = 0, z = 0"
    );
    ctx.assume("z != last").unwrap();
    assert!(ctx
        .check_index("z + 1", "first..=last")
        .unwrap()
        .is_proved());
}

#[test]
fn an_empty_domain_has_no_first() {
    // 5.6 case 23: V.first when V is empty; a one-value domain has first == last.
    let mut ctx = Context::new();
    ctx.var("lo", "Int").unwrap();
    ctx.var("hi", "Int").unwrap();
    assert_eq!(
        fails_for(&ctx.check_index("lo", "lo..=hi").unwrap()),
        "lo = 1, hi = 0"
    );
    ctx.assume("lo <= hi").unwrap();
    assert!(ctx.check_index("lo", "lo..=hi").unwrap().is_proved());
    assert!(ctx.check_index("hi", "lo..=hi").unwrap().is_proved());
}

#[test]
fn a_domain_holds_at_most_2_31_values() {
    // 5.6 case 19: domain V = 0..=3000000000.
    let mut ctx = Context::new();
    assert_eq!(
        fails_for(&ctx.prove("3000000000 - 0 + 1 <= 2 ** 31 - 1").unwrap()),
        "every input"
    );
    ctx.var("lo", "-1e9..=1e9").unwrap();
    ctx.var("hi", "-1e9..=1e9").unwrap();
    assert!(ctx
        .prove("max(hi - lo + 1, 0) <= 2 ** 31 - 1")
        .unwrap()
        .is_proved());
}

// ---- 4.2: slices -----------------------------------------------------------

#[test]
fn a_slice_of_a_slice_stays_inside_its_parent() {
    let mut ctx = Context::new();
    ctx.var("n", "0..=2e5").unwrap();
    ctx.var("s", "(lo: 0..=n, hi: 0..=n)").unwrap();
    ctx.assume("s.lo <= s.hi").unwrap();
    ctx.var("t", "(lo: Int, hi: Int)").unwrap();
    ctx.assume("s.lo <= t.lo and t.lo <= t.hi and t.hi <= s.hi")
        .unwrap();
    ctx.var("v", "Int").unwrap();
    ctx.assume("v in t.lo..t.hi").unwrap();
    assert!(ctx
        .prove("v in s.lo..s.hi and v in 0..n")
        .unwrap()
        .is_proved());
    // A window one past the parent's end is not inside it.
    assert_eq!(
        fails_for(&ctx.prove("v + 1 in s.lo..s.hi").unwrap()),
        "n = 1, s = (lo: 0, hi: 1), t = (lo: 0, hi: 1), v = 0"
    );
}

#[test]
fn a_backwards_range_is_the_empty_slice() {
    // a[z..=f] written backwards: its length is 0, never negative.
    let mut ctx = Context::new();
    ctx.var("z", "0..100").unwrap();
    ctx.var("f", "0..100").unwrap();
    ctx.assume("f < z").unwrap();
    assert!(ctx.prove("max(f + 1 - z, 0) == 0").unwrap().is_proved());
}

// ---- 4.3: grids -------------------------------------------------------------

#[test]
fn a_grid_cell_is_inside_the_flat_buffer() {
    let mut ctx = Context::new();
    ctx.var("H", "1..=1000").unwrap();
    ctx.var("W", "1..=1000").unwrap();
    ctx.var("r", "0..H").unwrap();
    ctx.var("c", "0..W").unwrap();
    assert!(ctx
        .check_index("r * W + c", "0..H * W")
        .unwrap()
        .is_proved());
    // The flat index determines the cell.
    assert!(ctx
        .prove("(r * W + c) / W == r and (r * W + c) % W == c")
        .unwrap()
        .is_proved());
    // The last cell is the last index: nothing is left over.
    assert_eq!(
        fails_for(&ctx.prove("r * W + c < H * W - 1").unwrap()),
        "H = 1, W = 1, r = 0, c = 0"
    );
    // Column-major by mistake overruns a non-square grid.
    let wrong = ctx.check_index("c * W + r", "0..H * W").unwrap();
    let counterexample = wrong.counterexample().expect("a counterexample");
    let get = |name: &str| counterexample.int(name).unwrap();
    assert!(get("c") * get("W") + get("r") >= get("H") * get("W"));
}

#[test]
fn a_neighbour_needs_a_bounds_check() {
    let mut ctx = Context::new();
    ctx.var("H", "1..=1000").unwrap();
    ctx.var("r", "0..H").unwrap();
    ctx.var("dr", "-1..=1").unwrap();
    assert_eq!(
        fails_for(&ctx.check_index("r + dr", "0..H").unwrap()),
        "H = 1, r = 0, dr = 1"
    );
    ctx.assume("0 <= r + dr and r + dr < H").unwrap();
    assert!(ctx.check_index("r + dr", "0..H").unwrap().is_proved());
}

#[test]
fn a_lazy_segment_tree_reaches_only_its_own_nodes() {
    // len == 2 * size; a node p < size has children 2p + 1 and 2p + 2.
    let mut ctx = Context::new();
    ctx.var("size", "1..=262144").unwrap();
    ctx.var("len", "Int").unwrap();
    ctx.assume("len == 2 * size").unwrap();
    ctx.var("p", "0..size - 1").unwrap();
    assert!(ctx.check_index("2 * p + 1", "0..len").unwrap().is_proved());
    assert!(ctx.check_index("2 * p + 2", "0..len").unwrap().is_proved());
    assert!(ctx
        .prove("(2 * p + 1 - 1) / 2 == p and (2 * p + 2 - 1) / 2 == p")
        .unwrap()
        .is_proved());
}

// ---- 5.6 cases 20 and 21: arithmetic at the ends of Int ----------------------

#[test]
fn remainders_are_never_negative() {
    let mut ctx = Context::new();
    ctx.var("a", "Int").unwrap();
    ctx.var("b", "Int").unwrap();
    assert!(ctx.prove("a % -1 == 0").unwrap().is_proved());
    assert!(ctx
        .prove("a % 7 in 0..7 and a % -7 in 0..7")
        .unwrap()
        .is_proved());
    // a / 0 and a % 0 stop the program unless the zero is ruled out.
    let unguarded = ctx.check_traps("a % b").unwrap();
    assert_eq!(unguarded.len(), 1);
    assert_eq!(unguarded[0].trouble, Trouble::DivisionByZero);
    assert_eq!(
        unguarded[0].to_string(),
        "`a % b` can divide by zero: a = 0, b = 0"
    );
    ctx.assume("b != 0").unwrap();
    assert!(ctx
        .check_traps("a % b")
        .unwrap()
        .iter()
        .all(|finding| finding.verdict.is_proved()));
    assert!(ctx
        .prove("0 <= a % b and a % b < abs(b)")
        .unwrap()
        .is_proved());
    assert!(ctx.prove("a == (a / b) * b + a % b").unwrap().is_proved());
}

#[test]
fn unbounded_arithmetic_can_overflow() {
    let mut ctx = Context::new();
    ctx.var("a", "Int").unwrap();
    ctx.var("k", "0..=62").unwrap();
    let report = |ctx: &mut Context, expr: &str| {
        ctx.check_overflow(expr)
            .unwrap()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    };
    assert_eq!(
        report(&mut ctx, "a + a"),
        "`a + a` can overflow Int: a = 4611686018427387904"
    );
    assert_eq!(
        report(&mut ctx, "a * a"),
        "`a * a` can overflow Int: a = 3037000500"
    );
    // Int.MIN is -Int.MAX, so negation and abs always fit.
    assert_eq!(report(&mut ctx, "-a"), "");
    assert_eq!(report(&mut ctx, "abs(a)"), "");
    assert!(report(&mut ctx, "a << k").starts_with("`a << k` can overflow Int: "));
    // A guard on the operands is a proof.
    ctx.assume("abs(a) <= 1e9").unwrap();
    assert_eq!(report(&mut ctx, "a + a"), "`a + a` cannot overflow Int");
    assert_eq!(report(&mut ctx, "a * a"), "`a * a` cannot overflow Int");
}
