//! Which DP optimization applies? Feed the transition cost (or the tables a brute force
//! filled) to these checks on small inputs before writing the fast version. Each check
//! returns `none` when the property held everywhere it looked, or the first place it
//! failed, so a wrong guess comes with its counterexample. `std.spec` has the checks on
//! a single cost function (`monge_break`, `dnc_break`, `knuth_break`,
//! `convexity_break`); this module runs the brute-force DPs themselves, reads tables,
//! finds periods, and names the optimizations that fit.
use std.spec.{decrease, convexity_break, concavity_break, monge_break}

/// A table read as a function, for the checks that take one: `at(t)(i, j) == t[i][j]`.
pub fn at(t: List<List<Int>>) -> (Int, Int) -> Int = (i, j) => t[i][j]

// ---- splitting a prefix into groups: dp[g][j] = min over i < j of dp[g-1][i] + cost(i, j)

/// The quadratic DP that splits `0..n` into consecutive groups, where the group of
/// boundaries `i..j` costs `cost(i, j)`. Returns `(dp, opt)`: `dp[g][j]` is the least
/// cost of splitting `0..j` into exactly `g` groups (`none` if impossible, `g` from 0 to
/// `groups`), and `opt[g][j]` the smallest `i` achieving it. O(groups * n^2) calls.
pub fn split_dp(cost: (Int, Int) -> Int, n: Int, groups: Int) -> (List<List<?Int>>, List<List<Int>>):
    var dp: List<List<?Int>> = [[none for _ in 0..=n] for _ in 0..=groups]
    var opt = [[0 for _ in 0..=n] for _ in 0..=groups]
    dp[0][0] = 0
    for g in 1..=groups:
        for j in 1..=n:
            for i in 0..j:
                if let prev = dp[g - 1][i]:
                    let v = prev + cost(i, j)
                    if dp[g][j] == none or v < dp[g][j]!:
                        dp[g][j] = v
                        opt[g][j] = i
    (dp, opt)

/// The first `(g, j)` where the best split point moves left as `j` grows within layer
/// `g` of `split_dp`, or `none`: then each layer can be filled by divide and conquer in
/// O(n log n) (`lib.dp.dnc_opt`).
pub fn split_opt_break(cost: (Int, Int) -> Int, n: Int, groups: Int) -> ?(Int, Int):
    let (dp, opt) = split_dp(cost, n, groups)
    for g in 1..=groups:
        var last = 0
        for j in 1..=n:
            if dp[g][j] == none: continue
            if opt[g][j] < last: return (g, j)
            last = opt[g][j]
    none

/// The best total cost of splitting `0..n` into exactly `g` groups, for each feasible
/// `g` in `1..=groups`, in order of `g`.
pub fn split_costs(cost: (Int, Int) -> Int, n: Int, groups: Int) -> List<Int>:
    let (dp, _) = split_dp(cost, n, groups)
    [dp[g][n]! for g in 1..=groups if dp[g][n] != none]

/// The first index where the best cost stops being convex in the number of groups, or
/// `none`: then "exactly k groups" can be solved by a penalty per group and a binary
/// search on it (Aliens trick / WQS).
pub fn aliens_break(cost: (Int, Int) -> Int, n: Int, groups: Int) -> ?Int:
    convexity_break(split_costs(cost, n, groups))

// ---- interval costs: cost(i, j) for boundaries 0 <= i < j <= n

/// The first `(i, j)` with `i < j` where the quadrangle inequality fails on intervals:
/// `cost(i, j) + cost(i + 1, j + 1) > cost(i, j + 1) + cost(i + 1, j)`. Only real
/// intervals are evaluated (`i + 1 < j + 1` and `i + 1 <= j`; an empty interval
/// `cost(j, j)` is read where it meets the diagonal, so define it, usually as 0).
pub fn interval_qi_break(cost: (Int, Int) -> Int, n: Int) -> ?(Int, Int):
    for i in 0..n:
        for j in i + 1..n:
            if cost(i, j) + cost(i + 1, j + 1) > cost(i, j + 1) + cost(i + 1, j): return (i, j)
    none

/// The first `(i, j)` where a longer interval is cheaper than one inside it:
/// `cost(i + 1, j) > cost(i, j)` or `cost(i, j - 1) > cost(i, j)`. `none` means `cost`
/// is monotone on inclusion.
pub fn inclusion_break(cost: (Int, Int) -> Int, n: Int) -> ?(Int, Int):
    for i in 0..n:
        for j in i + 2..=n:
            if cost(i + 1, j) > cost(i, j) or cost(i, j - 1) > cost(i, j): return (i, j)
    none

/// The cubic interval DP `dp[i][j] = cost(i, j) + min over i < k < j of dp[i][k] +
/// dp[k][j]` (unit intervals cost nothing), returning `(dp, opt)` with `opt[i][j]` the
/// smallest best `k` (and `opt[i][i + 1] = i`). O(n^3).
pub fn interval_dp(cost: (Int, Int) -> Int, n: Int) -> (List<List<Int>>, List<List<Int>>):
    var dp = [[0 for _ in 0..=n] for _ in 0..=n]
    var opt = [[0 for _ in 0..=n] for _ in 0..=n]
    for i in 0..n: opt[i][i + 1] = i
    for len in 2..=n:
        for i in 0..=n - len:
            let j = i + len
            var best = i + 1
            for k in i + 2..j:
                if dp[i][k] + dp[k][j] < dp[i][best] + dp[best][j]: best = k
            dp[i][j] = dp[i][best] + dp[best][j] + cost(i, j)
            opt[i][j] = best
    (dp, opt)

/// The first `(i, j)` where the best split of `i..j` in `interval_dp` leaves the range
/// `opt[i][j - 1] ..= opt[i + 1][j]`, or `none`: then Knuth's optimization
/// (`lib.dp.knuth`) fills the table in O(n^2). This tests the conclusion directly, so
/// it also catches costs that are Knuth-friendly without satisfying the textbook
/// conditions.
pub fn interval_opt_break(cost: (Int, Int) -> Int, n: Int) -> ?(Int, Int):
    let (_, opt) = interval_dp(cost, n)
    for i in 0..n:
        for j in i + 3..=n:
            if opt[i][j - 1] > opt[i][j] or opt[i][j] > opt[i + 1][j]: return (i, j)
    none

// ---- tables

/// The first `(i, j)` where a row of `t` stops being convex (at `t[i][j]`, `t[i][j + 1]`,
/// `t[i][j + 2]`), or `none`. Convex rows are what slope trick and a convex hull of
/// lines keep.
pub fn row_convexity_break(t: List<List<Int>>) -> ?(Int, Int):
    for i in 0..t.len:
        if let j = convexity_break(t[i]): return (i, j)
    none

/// The mirror image of `row_convexity_break`.
pub fn row_concavity_break(t: List<List<Int>>) -> ?(Int, Int):
    for i in 0..t.len:
        if let j = concavity_break(t[i]): return (i, j)
    none

/// How many earlier rows a table DP must keep. `reads` lists `(row written, row read)`
/// pairs logged by the brute force; the answer is the furthest distance back, so a
/// rolling buffer of that many rows plus the current one is enough. `none` if some
/// read looks at a later row, which a rolling buffer cannot serve. Reads within the
/// current row count as distance 0.
pub fn rolling_depth(reads: List<(Int, Int)>) -> ?Int:
    var depth = 0
    for (to, from) in reads:
        if from > to: return none
        depth = max(depth, to - from)
    depth

/// The same question asked of the table alone, for a DP whose row `i` is computed by
/// `step` from the `k` rows before it: the smallest `k <= max_depth` for which
/// `step(t[i - k .. i]) == t[i]` on every row from `k` on, or `none`. `step` receives
/// the `k` previous rows, oldest first.
pub fn rolling_window(t: List<List<Int>>, max_depth: Int, step: (List<List<Int>>) -> List<Int>) -> ?Int:
    for k in 1..=min(max_depth, t.len - 1):
        if all(step([t[r] for r in i - k..i]) == t[i] for i in k..t.len): return k
    none

// ---- periods

/// The eventual period of `xs`: the pair `(start, period)` with `xs[i] == xs[i + period]`
/// for every `i >= start`, choosing the smallest `start + period` (the fewest values to
/// store), then the smallest `period`. The periodic part must be seen at least twice in
/// full (`start + 2 * period <= xs.len`); `none` otherwise. Game values, Grundy numbers
/// and DP rows modulo something often repeat this way. O(n^2).
pub fn period<T>(xs: List<T>) -> ?(Int, Int):
    let n = xs.len
    var best: ?(Int, Int) = none
    for p in 1..=n / 2:
        // the longest suffix on which shifting by p changes nothing
        var start = n - p
        while start > 0 and xs[start - 1] == xs[start - 1 + p]: start -= 1
        if start + 2 * p > n: continue
        if best == none or start + p < best!.0 + best!.1: best = (start, p)
    best

/// The value at any index `n` of a sequence that repeats with `period(xs)`; `none` if
/// no period was confirmed.
pub fn periodic_at<T>(xs: List<T>, n: Int) -> ?T:
    if n < xs.len: return xs[n]
    if let (start, p) = period(xs): xs[start + (n - start) % p] else: none

// ---- naming the optimization

/// What a brute-force look at a split DP (`dp[g][j] = min over i < j of dp[g-1][i] +
/// cost(i, j)`, for `j <= n` and up to `groups` groups) suggests, one line per
/// optimization that fits. An empty list means none of the standard ones do.
pub fn split_report(cost: (Int, Int) -> Int, n: Int, groups: Int) -> List<Str>:
    var out = List<Str>()
    let monge = interval_qi_break(cost, n) == none
    if monge: out.push("quadrangle inequality holds: cost is Monge")
    if split_opt_break(cost, n, groups) == none:
        out.push("split points are monotone: divide and conquer, O(k n log n)")
    if aliens_break(cost, n, groups) == none and split_costs(cost, n, groups).len >= 3:
        out.push("cost is convex in the number of groups: Aliens trick")
    out

/// What a brute-force look at an interval DP (`dp[i][j] = cost(i, j) + min over k of
/// dp[i][k] + dp[k][j]`) suggests.
pub fn interval_report(cost: (Int, Int) -> Int, n: Int) -> List<Str>:
    var out = List<Str>()
    let qi = interval_qi_break(cost, n) == none
    let mono = inclusion_break(cost, n) == none
    if qi: out.push("quadrangle inequality holds")
    if mono: out.push("cost is monotone on inclusion")
    if interval_opt_break(cost, n) == none:
        if qi and mono: out.push("split points are monotone, as the theory promises: Knuth, O(n^2)")
        else: out.push("split points are monotone on this input only: Knuth is not guaranteed")
    out

@test
fn squared_group_sums_fit_everything():
    // split an array into k groups minimizing the sum of squared group sums
    for _ in 0..10:
        let n = rand(1, 12)
        let a = [rand(0, 9) for _ in 0..n]
        var pre = [0]
        for x in a: pre.push(pre.last + x)
        let cost = (i, j) => (pre[j] - pre[i]) ** 2
        let k = min(n, 5)
        assert interval_qi_break(cost, n) == none
        assert split_opt_break(cost, n, k) == none
        assert aliens_break(cost, n, k) == none
        let (dp, opt) = split_dp(cost, n, k)
        // spot-check the table against a direct minimum
        for g in 1..=k:
            for j in g..=n:
                assert dp[g][j]! == min(dp[g - 1][i]! + cost(i, j) for i in g - 1..j if dp[g - 1][i] != none)!
                assert dp[g - 1][opt[g][j]]! + cost(opt[g][j], j) == dp[g][j]!
        assert dp[0][0] == 0 and dp[0][n] == none and dp[1][0] == none
    let pre = [0, 3, 4, 9, 11]
    assert split_costs((i, j) => (pre[j] - pre[i]) ** 2, 4, 4) == [121, 65, 45, 39]
    assert split_report((i, j) => (pre[j] - pre[i]) ** 2, 4, 4).len == 3

@test
fn a_cost_that_fits_nothing_is_caught_with_a_witness():
    // rewarding long groups: sqrt-like concave costs break the quadrangle inequality
    let cost = (i, j) => -((j - i) ** 2)
    assert interval_qi_break(cost, 6) == (0, 1)
    assert aliens_break(cost, 6, 6) != none
    // split points jump left: the group 1..5 is free, so the last prefix splits early
    let jumpy = (i, j) => if i == 0: 100 elif (i, j) == (1, 5): 0 else: 5 - i
    assert split_opt_break(jumpy, 5, 2) == (2, 5)
    assert interval_qi_break(jumpy, 5) == (1, 4)
    assert split_report(jumpy, 5, 2) == []

@test
fn merging_piles_is_knuth_eligible():
    for _ in 0..10:
        let n = rand(1, 10)
        let a = [rand(0, 20) for _ in 0..n]
        var pre = [0]
        for x in a: pre.push(pre.last + x)
        let cost = (i, j) => pre[j] - pre[i]
        assert interval_qi_break(cost, n) == none and inclusion_break(cost, n) == none
        assert interval_opt_break(cost, n) == none
        let (dp, opt) = interval_dp(cost, n)
        for i in 0..n:
            for j in i + 2..=n:
                assert dp[i][j] == min(dp[i][k] + dp[k][j] for k in i + 1..j)! + cost(i, j)
                assert i < opt[i][j] and opt[i][j] < j
    let pre = [0, 4, 5, 6, 10]
    let cost = (i, j) => pre[j] - pre[i]
    assert interval_dp(cost, 4).0[0][4] == 18
    assert interval_report(cost, 4).len == 3 and interval_report(cost, 4)[2] == "split points are monotone, as the theory promises: Knuth, O(n^2)"

@test
fn interval_costs_that_break_knuth():
    // longer is cheaper: not monotone on inclusion
    let shrinking = (i, j) => 10 - (j - i)
    assert inclusion_break(shrinking, 5) == (0, 2)
    assert interval_qi_break(shrinking, 5) == none
    // a bonus for one specific interval breaks the quadrangle inequality next to it
    let spike = (i, j) => if (i, j) == (1, 3): 50 else: j - i
    assert interval_qi_break(spike, 5) == (0, 2)
    assert inclusion_break(spike, 5) == (0, 3)
    assert not interval_report(spike, 5).contains("quadrangle inequality holds")

@test
fn tables_read_as_functions():
    let t = [[(i - j) ** 2 for j in 0..6] for i in 0..6]
    assert monge_break(at(t), 6, 6) == none
    assert row_convexity_break(t) == none and row_concavity_break(t) == (0, 0)
    let bumpy = [[0, 1, 2], [0, 5, 2, 9]]
    assert row_convexity_break(bumpy) == (1, 0) and row_concavity_break(bumpy) == (1, 1)
    assert row_convexity_break(List<List<Int>>()) == none

@test
fn rolling_buffers():
    // knapsack reads only the previous row; LCS-style reads the previous and current
    assert rolling_depth([(1, 0), (2, 1), (2, 1), (3, 2)]) == 1
    assert rolling_depth([(2, 0), (2, 1), (3, 3)]) == 2
    assert rolling_depth([(1, 2)]) == none and rolling_depth(List<(Int, Int)>()) == 0
    // Pascal's triangle: every row follows from the one before
    var tri = [[1, 0, 0, 0, 0, 0]]
    for i in 1..6: tri.push([tri[i - 1][j] + (if j > 0: tri[i - 1][j - 1] else: 0) for j in 0..6])
    let pascal = (rows) => [rows.last[j] + (if j > 0: rows.last[j - 1] else: 0) for j in 0..6]
    assert rolling_window(tri, 3, pascal) == 1
    // row i = row i-1 + row i-2 elementwise: one row back is not enough
    var fibs = [[1, 2], [3, 5]]
    for i in 2..7: fibs.push([fibs[i - 1][j] + fibs[i - 2][j] for j in 0..2])
    let add2 = (rows) => [rows[rows.len - 1][j] + rows[rows.len - 2][j] for j in 0..2]
    let add1 = (rows) => [2 * rows.last[j] for j in 0..2]
    assert rolling_window(fibs, 1, add1) == none
    assert rolling_window(fibs, 3, (rows) => if rows.len < 2: [] else: add2(rows)) == 2

@test
fn periods_with_a_preperiod():
    assert period([1, 2, 3, 1, 2, 3, 1, 2]) == (0, 3)
    assert period([9, 9, 7, 0, 1, 0, 1, 0, 1]) == (3, 2)
    assert period([5, 5, 5, 5]) == (0, 1) and period([4, 5, 5]) == (1, 1)
    assert period([1, 2, 3, 4, 5, 6]) == none and period(List<Int>()) == none and period([1]) == none
    // seen once and a bit is not enough
    assert period([1, 2, 3, 1, 2]) == none
    for _ in 0..50:
        let (pre, p) = (rand(0, 5), rand(1, 6))
        let head = [rand(0, 2) for _ in 0..pre]
        let cycle = [rand(0, 2) for _ in 0..p]
        let xs = head + [cycle[i % p] for i in 0..3 * p]
        let (s, q) = period(xs)!
        assert q <= p and s <= pre
        for i in s..xs.len - q: assert xs[i] == xs[i + q]
        for n in [0, pre, xs.len, xs.len + 17, 10 ** 15]:
            let want = if n < pre: head[n] else: cycle[(n - pre) % p]
            assert periodic_at(xs, n) == want
    assert periodic_at([1, 2, 4], 9) == none and periodic_at([1, 2, 4], 2) == 4
    assert period([c for c in "abcabcab"]) == (0, 3) and period([c for c in "xababab"]) == (1, 2)
