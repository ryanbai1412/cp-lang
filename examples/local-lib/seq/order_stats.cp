//! Order-statistic sequence helpers: `ranks`, `lis`, and 2D offline `count_in_rects`.
use std.seq.{compress, first_ge_by}
use std.fenwick.{Fenwick}
use std.seq.{first_ge_by}

/// Dense 0-based sorted rank of every element in `xs`: equal elements get the same rank,
/// and `ranks([30, 10, 30, 20]) == [2, 0, 2, 1]`. O(n log n).
pub fn ranks<T>(xs: List<T>) -> List<Int>:
    if xs.is_empty: return []
    let cs = compress(xs)
    [cs.at(x).to_idx for x in xs]

/// Longest strictly increasing (or non-decreasing when `strict: false`) subsequence of `xs`.
/// Returns the sorted list of indices of one longest subsequence. O(n log n).
pub fn lis<T>(xs: List<T>, strict: Bool = true) -> List<Int>:
    let n = xs.len
    if n == 0: return []
    var tails_val: List<T> = []
    var tails_idx: List<Int> = []
    var prev = List.filled(n, -1)
    for i in 0..n:
        let x = xs[i]
        // Find first tail >= x (strict) or > x (non-decreasing)
        var (lo, hi) = (0, tails_val.len)
        while lo < hi:
            let mid = (lo + hi) / 2
            let go_left = if strict: tails_val[mid] >= x else: tails_val[mid] > x
            if go_left: hi = mid else: lo = mid + 1
        if lo > 0: prev[i] = tails_idx[lo - 1]
        if lo == tails_val.len:
            tails_val.push(x)
            tails_idx.push(i)
        else:
            tails_val[lo] = x
            tails_idx[lo] = i
    var res: List<Int> = []
    var cur = tails_idx.last!
    while cur != -1:
        res.push(cur)
        cur = prev[cur]
    var out = [res[i] for i in (0..res.len).rev]
    out

/// Count how many points `(x, y)` in `pts` lie inside each axis-aligned closed rectangle
/// `(x1, y1, x2, y2)` (`x1 <= x <= x2` and `y1 <= y <= y2`). Offline 2D Fenwick sweep in
/// O((n + q) log(n + q)).
pub fn count_in_rects(pts: List<(Int, Int)>, rects: List<(Int, Int, Int, Int)>) -> List<Int>:
    let q = rects.len
    if q == 0: return []
    if pts.is_empty: return List.filled(q, 0)
    var all_y = [p.1 for p in pts]
    for r in rects:
        all_y.push(r.1)
        all_y.push(r.3)
    let ys = compress(all_y)
    let spts = sorted(pts, by: p => p.0)
    // Events: (x, sign, y1_idx, y2_idx, query_id)
    var events: List<(Int, Int, Int, Int, Int)> = []
    for i in 0..q:
        let (x1, y1, x2, y2) = rects[i]
        if x1 > x2 or y1 > y2: continue
        let yi1 = ys.at(y1).to_idx
        let yi2 = ys.at(y2).to_idx
        events.push((x1 - 1, -1, yi1, yi2, i))
        events.push((x2, 1, yi1, yi2, i))
    sort(mut events, by: e => e.0)
    domain Y = 0..ys.len
    var bit = Fenwick<Y, Int>()
    var ans = List.filled(q, 0)
    var pi = 0
    for (x, sgn, y1, y2, qi) in events:
        while pi < spts.len and spts[pi].0 <= x:
            bit.add(Y(ys.at(spts[pi].1).to_idx), 1)
            pi += 1
        ans[qi] += sgn * bit.sum(Y(y1)..=Y(y2))
    ans

@test
fn ranks_and_lis_match_brute_force():
    assert ranks([30, 10, 30, 20]) == [2, 0, 2, 1]
    assert ranks::<Int>([]) == []
    for _ in 0..25:
        let n = rand(1, 14)
        let xs = [rand(0, 6) for _ in 0..n]
        // Check ranks order preservation
        let rk = ranks(xs)
        for i in 0..n:
            for j in 0..n:
                assert (rk[i] < rk[j]) == (xs[i] < xs[j])
                assert (rk[i] == rk[j]) == (xs[i] == xs[j])
        // Check strict and non-decreasing LIS against O(n^2) DP
        for strict in [true, false]:
            let idxs = lis(xs, strict: strict)
            var dp = List.filled(n, 1)
            for i in 0..n:
                for j in 0..i:
                    let ok = if strict: xs[j] < xs[i] else: xs[j] <= xs[i]
                    if ok and dp[j] + 1 > dp[i]: dp[i] = dp[j] + 1
            assert idxs.len == max(dp)
            for k in 1..idxs.len:
                assert idxs[k - 1] < idxs[k]
                if strict: assert xs[idxs[k - 1]] < xs[idxs[k]]
                else: assert xs[idxs[k - 1]] <= xs[idxs[k]]

@test
fn count_in_rects_matches_quadratic_scan():
    for _ in 0..20:
        let pts = [(rand(-10, 10), rand(-10, 10)) for _ in 0..rand(0, 25)]
        var rects: List<(Int, Int, Int, Int)> = []
        for _ in 0..rand(1, 15):
            let x1 = rand(-12, 12)
            let x2 = rand(-12, 12)
            let y1 = rand(-12, 12)
            let y2 = rand(-12, 12)
            rects.push((x1, y1, x2, y2))
        let got = count_in_rects(pts, rects)
        for i in 0..rects.len:
            let (x1, y1, x2, y2) = rects[i]
            let want = count(x1 <= p.0 and p.0 <= x2 and y1 <= p.1 and p.1 <= y2 for p in pts)
            assert got[i] == want
