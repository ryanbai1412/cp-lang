//! Search on answers: binary, exponential, ternary and parallel searches,
//! and fractional programming. Each returns a position of the space searched
//! or `none`, never a sentinel, and names the monotone or unimodal law it
//! needs. Binary search over a range is a method on it (`std.seq`):
//! `r.first_true(ok)`, `r.last_true(ok)`, `r.bsearch(ok, last:)`. Sorted lookup in a container is the four-name family
//! `first_ge`/`first_gt`/`last_le`/`last_lt`; a search inside a range
//! structure is its `max_right`/`min_left`.

/// `steps + 1` calls of `ok`.
/// Fixed-iteration binary search on `[lo, hi]` for monotone `ok` (false then true).
/// Returns the least `x` in `[lo, hi]` with `ok(x)` to within `(hi - lo) / 2^steps`, or `none`
/// when `not ok(hi)`. `steps + 1` calls of `ok`.
pub fn bsearch_real(lo: F64, hi: F64, ok: F64 -> Bool, steps: Int = 100) -> ?F64 = todo

/// O(log n) calls of `f`.
/// Position maximizing a discrete unimodal function `f` over `r`, or `none` if `r` is empty.
/// `f` must be strictly rising then falling (plateaus only at the optimum). O(log |r|) calls of `f`.
pub fn ternary_max<I, K>(r: Range<I>, f: I -> K) -> ?I = todo

/// O(log n) calls of `f`.
/// Position minimizing a discrete unimodal function `f` over `r`, or `none` if `r` is empty.
/// `f` must be strictly falling then rising (plateaus only at the optimum). O(log |r|) calls of `f`.
pub fn ternary_min<I, K>(r: Range<I>, f: I -> K) -> ?I = todo

/// `steps + 2` calls of `f`.
/// Point in `[lo, hi]` maximizing continuous unimodal `f` via golden-section search.
/// Within `(hi - lo) * 0.618^steps` of the optimum. `steps + 2` calls of `f`.
pub fn ternary_max_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64 = todo

/// `steps + 2` calls of `f`.
/// Point in `[lo, hi]` minimizing continuous unimodal `f` via golden-section search.
/// Within `(hi - lo) * 0.618^steps` of the optimum. `steps + 2` calls of `f`.
pub fn ternary_min_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64 = todo

/// O((n + queries) log n) calls of `apply` and `ok`, O(n + queries) memory.
/// Many binary searches over one timeline `r` at once. `reset()` empties the user's
/// structure, `apply(t)` adds the event at time `t`, and `ok(q)` asks query `q` of the
/// structure as it stands; `out[q]` is the first `t` with `ok(q)` once every event up to
/// `t` is applied, or `none`. Law: each `ok(q)` stays true once it holds. O(log n)
/// rounds, each one `reset`, `n` calls of `apply` and one `ok` per open query, for
/// n = `r.len`; memory O(n + queries).
pub fn parallel_bsearch<I>(
    queries: Int,
    r: Range<I>,
    reset: () -> (),
    apply: I -> (),
    ok: Int -> Bool,
) -> List<?I> = todo

/// O(log d) calls of `ok`, d the distance from `from` to the answer.
/// The first `i >= from` with `ok(i)`, doubling then bisecting, for a
/// monotone `ok` (false then true) with no upper end known; `none` when `ok`
/// stays false to the end of `I`.
pub fn gallop_first<I>(from: I, ok: I -> Bool) -> ?I = todo

/// O(log(1 / eps)) calls of `solve`; superlinear in practice.
/// The largest ratio `num / den` over the choices, where `solve(l)` returns
/// the choice maximizing `num - l * den`; `den > 0` for every choice and
/// `lo` at most the answer (Dinkelbach).
pub fn dinkelbach(lo: F64, solve: F64 -> (num: F64, den: F64), eps: F64 = 1e-9) -> F64 = todo
