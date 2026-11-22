//! Search on answers: binary, exponential, ternary and parallel searches,
//! and fractional programming. Each returns a position of the space searched
//! or `None`, never a sentinel, and names the monotone or unimodal law it
//! needs. Binary search over a range is a method on it (`std.seq`):
//! `r.first_true(ok)`, `r.last_true(ok)`, `r.bsearch(ok, last:)`. Sorted lookup in a container is the four-name family
//! `first_ge`/`first_gt`/`last_le`/`last_lt`; a search inside a range
//! structure is its `right_while`/`left_while`.

/// Fixed-iteration binary search on `[lo, hi]` for a monotone `ok` (false then true): the least
/// `x` with `ok(x)`, to within `(hi - lo) / 2^steps`, or `None` when `not ok(hi)`.
///
/// It first calls `ok(hi)`; then `steps` times it tests the midpoint `m` of `[lo, hi]` and keeps
/// `[lo, m]` when `ok(m)`, else `[m, hi]`. The result is the final `hi`, so `ok` holds there.
///
/// cost: `steps + 1` calls of `ok`
/// traps: when `lo > hi` or `steps < 0`
/// example: `bsearch_real(0.0, 8.0, x => x >= 3.0, steps: 3) == Some(3.0)`
/// example: `bsearch_real(0.0, 1.0, x => x > 2.0) == None`
/// example: `close(bsearch_real(0.0, 2.0, x => x * x >= 2.0)!, 1.4142135623730951)`
pub fn bsearch_real(lo: F64, hi: F64, ok: F64 -> Bool, steps: Int = 100) -> ?F64 = todo

/// The position of `r` where a discrete unimodal `f` is largest, or `None` when `r` is empty.
///
/// `f` must be strictly rising then strictly falling, with equal values allowed only on a
/// plateau at the maximum. Golden-section search: each probe is reused, so it makes about
/// log_phi |r| + 3 distinct calls of `f` (29 for 1e6), which is what interactive query budgets
/// assume. A non-unimodal `f` gives an unspecified position of `r`.
///
/// cost: O(log n) calls of `f`, about log_phi n + 3, for n = `r.len`
/// empty: `None`
/// ties: the first position of the plateau
/// traps: never
/// example: `ternary_max(0..6, i => -(i - 3) * (i - 3)) == Some(3)`
/// example: `let f = [0, 1, 2, 2, 1]; ternary_max(0..5, i => f[i]) == Some(2)`
/// example: `ternary_max(0..0, i => i) == None`
pub fn ternary_max<I, K>(r: Range<I>, f: I -> K) -> ?I = todo

/// The position of `r` where a discrete unimodal `f` is smallest, or `None` when `r` is empty.
///
/// `f` must be strictly falling then strictly rising, with equal values allowed only on a
/// plateau at the minimum. Golden-section search: each probe is reused, so it makes about
/// log_phi |r| + 3 distinct calls of `f` (29 for 1e6), which is what interactive query budgets
/// assume. A non-unimodal `f` gives an unspecified position of `r`.
///
/// cost: O(log n) calls of `f`, about log_phi n + 3, for n = `r.len`
/// empty: `None`
/// ties: the first position of the plateau
/// traps: never
/// example: `ternary_min(0..6, i => (i - 4) * (i - 4)) == Some(4)`
/// example: `let f = [3, 1, 1, 2]; ternary_min(0..4, i => f[i]) == Some(1)`
/// example: `ternary_min(5..5, i => i) == None`
pub fn ternary_min<I, K>(r: Range<I>, f: I -> K) -> ?I = todo

/// A point of `[lo, hi]` where a continuous unimodal `f` is largest, by golden-section search,
/// within `(hi - lo) * 0.618^steps` of the maximizer.
///
/// Two interior probes start the search; each of the `steps` rounds keeps the side holding the
/// larger probe and adds one probe. The result is the midpoint of the final bracket.
///
/// cost: `steps + 2` calls of `f`
/// ties: on a plateau at the maximum, whichever plateau point the bracket closes on
/// traps: when `lo > hi` or `steps < 0`
/// example: `close(ternary_max_real(0.0, 4.0, x => -(x - 1.0) * (x - 1.0)), 1.0, eps: 1e-6)`
/// example: `ternary_max_real(2.0, 2.0, x => x) == 2.0`
pub fn ternary_max_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64 = todo

/// A point of `[lo, hi]` where a continuous unimodal `f` is smallest, by golden-section search,
/// within `(hi - lo) * 0.618^steps` of the minimizer.
///
/// Two interior probes start the search; each of the `steps` rounds keeps the side holding the
/// smaller probe and adds one probe. The result is the midpoint of the final bracket.
///
/// cost: `steps + 2` calls of `f`
/// ties: on a plateau at the minimum, whichever plateau point the bracket closes on
/// traps: when `lo > hi` or `steps < 0`
/// example: `close(ternary_min_real(-3.0, 3.0, x => (x - 2.0) * (x - 2.0)), 2.0, eps: 1e-6)`
/// example: `ternary_min_real(1.5, 1.5, x => x) == 1.5`
pub fn ternary_min_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64 = todo

/// Many binary searches over one timeline `r` at once: `out[q]` is the first `t` of `r` such
/// that `ok(q)` holds once every event of `r` up to and including `t` is applied, or `None`
/// when it never holds.
///
/// `reset()` empties the caller's structure, `apply(t)` adds the event at time `t`, and `ok(q)`
/// asks query `q` of the structure as it stands. Law: once `ok(q)` holds it stays true as more
/// events are applied. Each of the O(log n) rounds calls `reset` once, `apply` on every `t` of
/// `r` in increasing order, and `ok` once per open query, between the right `apply` calls.
///
/// cost: O((n + queries) log n) calls of `apply` and `ok`, O(n + queries) memory, for n = `r.len`
/// empty: every entry `None` when `r` is empty; `[]` when `queries == 0`
/// order: `out[q]` answers query `q`, for `q` in `0..queries`
/// traps: when `queries < 0`
/// example: `let need = [2, 9]; var c = 0; parallel_bsearch(2, 0..4, reset: () => c = 0, apply: _ => c += 1, ok: q => c >= need[q]) == [Some(1), None]`
/// example: `parallel_bsearch(1, 0..0, reset: () => (), apply: _ => (), ok: _ => true) == [None]`
pub fn parallel_bsearch<I>(
    queries: Int,
    r: Range<I>,
    reset: () -> (),
    apply: I -> (),
    ok: Int -> Bool,
) -> List<?I> = todo

/// The first `i >= from` with `ok(i)`, for a monotone `ok` (false then true) whose upper end is
/// unknown, or `None` when `ok` stays false to the end of `I`.
///
/// Exponential search: probes `from`, `from + 1`, `from + 3`, `from + 7`, ... (doubling the step)
/// until `ok` holds or `I` ends, then bisects the last gap.
///
/// cost: O(log d) calls of `ok`, for d the distance from `from` to the answer
/// traps: never
/// example: `gallop_first(0, i => i * i >= 50) == Some(8)`
/// example: `gallop_first(10, i => i >= 3) == Some(10)`
pub fn gallop_first<I>(from: I, ok: I -> Bool) -> ?I = todo

/// The largest ratio `num / den` over a family of choices (Dinkelbach's method), where
/// `solve(l)` returns the choice maximizing `num - l * den`.
///
/// Requires `den > 0` for every choice and `lo` at most the answer. Starting from `l = lo`, it
/// calls `solve(l)`; when `num - l * den <= eps` it returns `l`, else it sets `l = num / den`
/// and repeats. `l` never decreases, and it converges superlinearly.
///
/// cost: O(log(1 / eps)) calls of `solve`, usually a handful
/// traps: when `solve` returns `den <= 0`
/// example: `dinkelbach(0.0, solve: _ => (num: 3.0, den: 2.0)) == 1.5`
/// example: `dinkelbach(0.0, solve: l => if l < 1.0: (num: 3.0, den: 2.0) else: (num: 2.0, den: 1.0)) == 2.0`
pub fn dinkelbach(lo: F64, solve: F64 -> (num: F64, den: F64), eps: F64 = 1e-9) -> F64 = todo

/// The values of a family of ranges, largest first, lazily. Each group `g` owns `range(g)`;
/// `best(g, r)` is the top value over a non-empty `r` and the position of `r` that gives it.
///
/// A queue holds one entry per non-empty group range. Taking the top entry `(g, r)` with value
/// `t` at position `p` yields `t` and queues the non-empty pieces `r.start..p` and `p.next..r.end`
/// with their own `best`. So the k best cost O(k log(n + k)) `best` calls, n the number of
/// groups: the k largest subarray sums, Super Piano, k smallest pair sums (with negated values).
///
/// cost: O(n + k) calls of `best` and O((n + k) log(n + k)) queue work for the first k values
/// empty: yields nothing when `groups` is empty or every range is empty
/// ties: equal values come out in group order, then left piece before right piece
/// order: decreasing `T`
/// traps: when `best` returns a position outside the range it was given
/// example: `let a = [3, 1, 2]; split_best([0..3], range: g => g, best: (_, r) => r.map(i => (a[i], i)).max()!).collect() == [3, 2, 1]`
/// example: `split_best([0..0], range: g => g, best: (_, r) => (0, r.start)).collect() == []`
pub fn split_best<G, D, T: Ord>(
    groups: Seq<G>,
    range: G -> Range<D>,
    best: (G, Range<D>) -> (T, D),
) -> yields T = todo
