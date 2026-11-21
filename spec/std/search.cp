//! Search on answers: binary, exponential, ternary and parallel searches,
//! and fractional programming. Each returns a position of the space searched
//! or `none`, never a sentinel, and names the monotone or unimodal law it
//! needs. Sorted lookup in a container is the four-name family
//! `first_ge`/`first_gt`/`last_le`/`last_lt`; a search inside a range
//! structure is its `max_right`/`min_left`.

/// O(log n) calls of `ok`; O(log d) from a `warm` guess at distance d
/// once implemented (today `warm` is accepted and ignored).
/// The first position of `r` where `ok` holds, or `none`. `ok` must be false and then true
/// along the range; the two ends are checked against that. O(log n) calls of `ok`.
pub fn bsearch_first<I>(r: Range<I>, ok: I -> Bool, warm: ?I = none) -> ?I:
    if r.is_empty: return none
    var lo = r.first!
    var hi = r.last!
    if not ok(hi): return none
    assert not (ok(lo) and not ok(hi))  // `ok` must not switch back to false
    while lo < hi:
        let mid = (lo + (hi - lo) / 2)!
        if ok(mid): hi = mid else: lo = (mid + 1)!
    lo

/// O(log n) calls of `ok`.
/// The last position of `r` where `ok` holds, or `none`. `ok` must be true and then false
/// along the range. O(log n) calls of `ok`.
pub fn bsearch_last<I>(r: Range<I>, ok: I -> Bool) -> ?I:
    if r.is_empty: return none
    var lo = r.first!
    var hi = r.last!
    if not ok(lo): return none
    while lo < hi:
        let span = hi - lo
        let mid = (lo + (span - span / 2))!
        if ok(mid): lo = mid else: hi = (mid - 1)!
    lo

/// `steps + 1` calls of `ok`.
/// Fixed-iteration binary search on `[lo, hi]` for monotone `ok` (false then true).
/// Returns the least `x` in `[lo, hi]` with `ok(x)` to within `(hi - lo) / 2^steps`, or `none`
/// when `not ok(hi)`. `steps + 1` calls of `ok`.
pub fn bsearch_real(lo: F64, hi: F64, ok: F64 -> Bool, steps: Int = 100) -> ?F64:
    if not ok(hi): return none
    var (l, r) = (lo, hi)
    for _ in 0..steps:
        let m = (l + r) * 0.5
        if ok(m): r = m else: l = m
    r

/// O(log n) calls of `f`.
/// Position maximizing a discrete unimodal function `f` over `r`, or `none` if `r` is empty.
/// `f` must be strictly rising then falling (plateaus only at the optimum). O(log |r|) calls of `f`.
pub fn ternary_max<I, K>(r: Range<I>, f: I -> K) -> ?I:
    if r.is_empty: return none
    var lo = r.first!
    var hi = r.last!
    while lo < hi:
        let mid = (lo + (hi - lo) / 2)!
        if f(mid) < f((mid + 1)!): lo = (mid + 1)! else: hi = mid
    lo

/// O(log n) calls of `f`.
/// Position minimizing a discrete unimodal function `f` over `r`, or `none` if `r` is empty.
/// `f` must be strictly falling then rising (plateaus only at the optimum). O(log |r|) calls of `f`.
pub fn ternary_min<I, K>(r: Range<I>, f: I -> K) -> ?I:
    if r.is_empty: return none
    var lo = r.first!
    var hi = r.last!
    while lo < hi:
        let mid = (lo + (hi - lo) / 2)!
        if f(mid) > f((mid + 1)!): lo = (mid + 1)! else: hi = mid
    lo

/// `steps + 2` calls of `f`.
/// Point in `[lo, hi]` maximizing continuous unimodal `f` via golden-section search.
/// Within `(hi - lo) * 0.618^steps` of the optimum. `steps + 2` calls of `f`.
pub fn ternary_max_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64:
    let inv_phi = 0.6180339887498949
    var (l, r) = (lo, hi)
    var m1 = r - (r - l) * inv_phi
    var m2 = l + (r - l) * inv_phi
    var f1 = f(m1)
    var f2 = f(m2)
    for _ in 0..steps:
        if f1 < f2:
            l = m1
            m1 = m2
            f1 = f2
            m2 = l + (r - l) * inv_phi
            f2 = f(m2)
        else:
            r = m2
            m2 = m1
            f2 = f1
            m1 = r - (r - l) * inv_phi
            f1 = f(m1)
    (l + r) * 0.5

/// `steps + 2` calls of `f`.
/// Point in `[lo, hi]` minimizing continuous unimodal `f` via golden-section search.
/// Within `(hi - lo) * 0.618^steps` of the optimum. `steps + 2` calls of `f`.
pub fn ternary_min_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64:
    let inv_phi = 0.6180339887498949
    var (l, r) = (lo, hi)
    var m1 = r - (r - l) * inv_phi
    var m2 = l + (r - l) * inv_phi
    var f1 = f(m1)
    var f2 = f(m2)
    for _ in 0..steps:
        if f1 > f2:
            l = m1
            m1 = m2
            f1 = f2
            m2 = l + (r - l) * inv_phi
            f2 = f(m2)
        else:
            r = m2
            m2 = m1
            f2 = f1
            m1 = r - (r - l) * inv_phi
            f1 = f(m1)
    (l + r) * 0.5

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
) -> List<?I>:
    let n = r.len
    var lo = List.filled(queries, 0)
    var hi = List.filled(queries, n)
    while true:
        var start = List.filled(n + 1, 0)
        for q in 0..queries:
            if lo[q] < hi[q]: start[(lo[q] + hi[q]) / 2 + 1] += 1
        if all(lo[q] == hi[q] for q in 0..queries): break
        for t in 0..n: start[t + 1] += start[t]
        var order = List.filled(start[n], 0)
        var fill = [x for x in start]
        for q in 0..queries:
            if lo[q] < hi[q]:
                let t = (lo[q] + hi[q]) / 2
                order[fill[t]] = q
                fill[t] += 1
        reset()
        for t in 0..n:
            apply((r.first! + t)!)
            for at in start[t]..start[t + 1]:
                let q = order[at]
                if ok(q): hi[q] = t else: lo[q] = t + 1
    [if lo[q] == n: none else: (r.first! + lo[q])! for q in 0..queries]

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
