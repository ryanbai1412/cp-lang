//! The DP toolkit: the structures and optimizations a DP calls by name.
//!
//! A minimum or maximum of lines (the convex hull trick) is a `LiChao`; a
//! layer whose optimal split moves right is `dnc_layer` (over `monotone_minima`); an interval DP with
//! monotone splits is `knuth`; "exactly `k` pieces" over a convex cost is
//! `aliens`. Meet in the middle starts from `subset_sums`; sums over subsets and
//! supersets of masks are `std.ntt`'s `sos_sub` / `sos_sup`. The
//! recurrence itself stays in your code, and a memoized recursion is
//! `@memo`. Each optimization names the law it needs; `std.spec` checks it
//! on small inputs (`monge_break`, `dnc_break`, `knuth_break`,
//! `convexity_break`). A sliding-window optimum is `std.window.MonoDeque`.

use std.seq.{bsearch_first}

/// The line `k * x + m`, the usual entry of a `LiChao`.
pub struct Line(k: Int, m: Int):
    /// Evaluate the line `k * x + m` at `x`. O(1).
    pub fn at(self, x: Int) -> Int = k * x + m

/// The lower (`LiChao<Line, Int>(min, 0..=1e9)`) or upper (`max`) envelope
/// of lines over the integer coordinates `xs`. A line is anything with
/// `at(self, x: Int) -> K`, so a struct whose `at` returns `(cost, used)`
/// breaks ties by its second field with the same tree. Law: two lines cross
/// at most once on `xs`. `add` is O(log n) for a whole line and O(log^2 n)
/// on a segment (`on: lo..=hi`), `at` and `line_at` O(log n), for n =
/// `xs.len`; memory is O(lines) cells, allocated as lines arrive.
pub struct LiChao<L, K>(pick: (K, K) -> K, lo: Int, hi: Int):
    lines: List<?L>
    // a cell's children; 0 is none, since the root is no cell's child
    left: List<Int>
    right: List<Int>

    /// Create a Li Chao tree over integer x-domain `xs` with comparator `pick`. O(1).
    pub fn new(pick: (K, K) -> K, xs: Range<Int>) -> LiChao<L, K>:
        assert not xs.is_empty
        LiChao(pick, xs.first!, xs.last!, [none], [0], [0])

    /// Forget every line. O(1) amortized.
    pub fn clear(mut self):
        (lines, left, right) = ([none], [0], [0])

    /// The best value of a line at `x`, or `none` when no line covers `x`.
    pub fn at(self, x: Int) -> ?K:
        assert lo <= x and x <= hi
        var best: ?K = none
        var (i, l, r) = (0, lo, hi)
        unchecked:
            while true:
                if let c = lines[i]:
                    let v = c.at(x)
                    best = if let b = best: pick(b, v) else: v
                let mid = l + (r - l) / 2
                if x <= mid:
                    i = left[i]
                    r = mid
                else:
                    i = right[i]
                    l = mid + 1
                if i == 0: break
        best

    /// The line that is best at `x`, or `none` when no line covers `x`.
    pub fn line_at(self, x: Int) -> ?L:
        assert lo <= x and x <= hi
        var best: ?(line: L, val: K) = none
        var (i, l, r) = (0, lo, hi)
        while true:
            if let c = lines[i]:
                let v = c.at(x)
                if best == none or self.wins(v, best!.val): best = (line: c, val: v)
            let mid = l + (r - l) / 2
            if x <= mid:
                i = left[i]
                r = mid
            else:
                i = right[i]
                l = mid + 1
            if i == 0: break
        if let b = best: b.line else: none

    /// Add `line` on all of `xs`, or only on the coordinates `on`.
    pub fn add(mut self, line: L, on: ?Range<Int> = none):
        if on == none:
            self.settle(0, lo, hi, line)
            return
        let s = on!
        if s.is_empty: return
        let (a, b) = (max(s.first!, lo), min(s.last!, hi))
        if a > b: return
        var todo = [(0, lo, hi)]
        while let t = todo.pop():
            let (i, l, r) = t
            if a <= l and r <= b:
                self.settle(i, l, r, line)
                continue
            let mid = l + (r - l) / 2
            if a <= mid: todo.push((self.kid(i, false), l, mid))
            if mid < b: todo.push((self.kid(i, true), mid + 1, r))

    fn wins(self, a: K, b: K) -> Bool = pick(a, b) == a and a != b

    fn kid(mut self, i: Int, right_side: Bool) -> Int:
        let k = if right_side: right[i] else: left[i]
        if k != 0: return k
        lines.push(none)
        left.push(0)
        right.push(0)
        let fresh = lines.len - 1
        if right_side: right[i] = fresh else: left[i] = fresh
        fresh

    // Keep the line better at the middle of `l..=r` here; the other one is
    // better on at most one half, and goes down into it.
    fn settle(mut self, at: Int, lo: Int, hi: Int, line: L):
        var (i, l, r, fresh) = (at, lo, hi, line)
        unchecked:
            while true:
                if lines[i] == none:
                    lines[i] = fresh
                    return
                let old = lines[i]!
                let mid = l + (r - l) / 2
                let left_wins = self.wins(fresh.at(l), old.at(l))
                let mid_wins = self.wins(fresh.at(mid), old.at(mid))
                if mid_wins:
                    lines[i] = fresh
                    fresh = old
                if l == r: return
                if left_wins != mid_wins:
                    i = self.kid(i, false)
                    r = mid
                else:
                    i = self.kid(i, true)
                    l = mid + 1

/// For each row, the least `f(row, col)` and its leftmost `col`, or `none`
/// where `f` is `none` on the whole row. Law: the leftmost argmin does not
/// decrease from row to row (true when `f` is Monge; `std.spec.monge_break`
/// checks it). O((rows + cols) log rows) calls of `f`.
pub fn monotone_minima(rows: Int, cols: Int, f: (Int, Int) -> ?Int) -> List<?(val: Int, at: Int)>:
    var out: List<?(val: Int, at: Int)> = List.filled(rows, none)
    var todo = [(0, rows, 0, cols)]
    while let t = todo.pop():
        let (lo, hi, from, to) = t
        if lo >= hi: continue
        let mid = (lo + hi) / 2
        var best: ?(val: Int, at: Int) = none
        for j in from..to:
            if let v = f(mid, j):
                if best == none or v < best!.val: best = (val: v, at: j)
        out[mid] = best
        let cut = if let b = best: b.at else: from
        todo.push((lo, mid, from, if best == none: to else: cut + 1))
        todo.push((mid + 1, hi, cut, to))
    out

/// One layer of a partition DP: `next[i]` is the least `prev[j] + cost(j, i)`
/// over `j < i` with `prev[j]` present, so `next[0]` is `none`. Indices are
/// cuts `0..prev.len`. Law: `cost` satisfies the quadrangle inequality
/// (`std.spec.dnc_break` checks it), so the best `j` moves right with `i`.
/// O(n log n) calls of `cost` for n = `prev.len`.
pub fn dnc_layer(prev: List<?Int>, cost: (Int, Int) -> Int) -> List<?Int>:
    let n = prev.len
    var next: List<?Int> = List.filled(n, none)
    var todo = [(1, n, 0, n)]
    unchecked:
        while let t = todo.pop():
            let (lo, hi, from, to) = t
            if lo >= hi: continue
            let mid = (lo + hi) / 2
            var best: ?Int = none
            var at = from
            for j in from..min(to, mid):
                if let p = prev[j]:
                    let v = p + cost(j, mid)
                    if best == none or v < best!:
                        best = v
                        at = j
            next[mid] = best
            todo.push((lo, mid, from, if best == none: to else: at + 1))
            todo.push((mid + 1, hi, at, to))
    next

/// The interval DP over cuts `0..=n`: `dp[i][i + 1] = 0` and `dp[i][j] =
/// cost(i, j) + min over i < k < j of dp[i][k] + dp[k][j]`. Law: `cost` is
/// monotone under inclusion and satisfies the quadrangle inequality
/// (`std.spec.knuth_break` checks the split monotonicity it implies), so the
/// best split of `[i, j)` lies between those of `[i, j - 1)` and `[i + 1, j)`.
/// O(n^2) time and memory.
pub fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>>:
    var dp: List<List<Int>> = []
    var opt: List<List<Int>> = []
    for _ in 0..=n:
        dp.push(List.filled(n + 1, 0))
        opt.push(List.filled(n + 1, 0))
    for i in 0..n: opt[i][i + 1] = i
    unchecked:
        for len in 2..=n:
            for i in 0..=n - len:
                let j = i + len
                let from = max(opt[i][j - 1], i + 1)
                let to = max(from, min(opt[i + 1][j], j - 1))
                var best = dp[i][from] + dp[from][j]
                var at = from
                for k in from + 1..=to:
                    let v = dp[i][k] + dp[k][j]
                    if v < best:
                        best = v
                        at = k
                dp[i][j] = best + cost(i, j)
                opt[i][j] = at
    dp

/// `f(k)` for a convex `f` known only through its penalized form:
/// `solve(λ)` is the least `f(c) + λ * c` over every count `c`, and the least
/// `c` attaining it. Law: `f` is convex (`std.spec.convexity_break`), and the
/// first `λ` of `lambdas` gives a count of at most `k` somewhere in the range
/// (the counts fall as `λ` grows). O(log lambdas.len) calls of `solve`.
pub fn aliens(k: Int, lambdas: Range<Int>, solve: Int -> (val: Int, count: Int)) -> Int:
    let lam = bsearch_first(lambdas, l => solve(l).count <= k)!
    solve(lam).val - lam * k

/// Every subset sum of `xs`, sorted: the half of a meet-in-the-middle.
/// O(2^n) time and memory, by merging.
pub fn subset_sums(xs: List<Int>) -> List<Int>:
    var out = [0]
    for x in xs:
        var next: List<Int> = []
        var (i, j) = (0, 0)
        while i < out.len or j < out.len:
            if j == out.len or i < out.len and out[i] <= out[j] + x:
                next.push(out[i])
                i += 1
            else:
                next.push(out[j] + x)
                j += 1
        out = next
    out

struct Counted(k: Int, m: Int, used: Int):
    fn at(self, x: Int) -> (cost: Int, used: Int) = (cost: k * x + m, used: used)

@test
fn li_chao_matches_a_scan_over_lines_and_segments():
    for _ in 0..100:
        let (lo, hi) = (rand(-20, 0), rand(1, 20))
        var low = LiChao<Line, Int>(min, lo..=hi)
        var high = LiChao<Line, Int>(max, lo..=hi)
        var lines: List<(l: Line, a: Int, b: Int)> = []
        for _ in 0..rand(1, 12):
            let l = Line(rand(-5, 5), rand(-20, 20))
            if rand(0, 1) == 0:
                low.add(l)
                high.add(l)
                lines.push((l: l, a: lo, b: hi))
            else:
                let (a, b) = (rand(lo - 3, hi), rand(lo, hi + 3))
                low.add(l, on: a..=b)
                high.add(l, on: a..=b)
                lines.push((l: l, a: a, b: b))
            for x in lo..=hi:
                let here = [e.l.at(x) for e in lines if e.a <= x and x <= e.b]
                assert low.at(x) == min(here)
                assert high.at(x) == max(here)
    var ties = LiChao<Counted, (cost: Int, used: Int)>(min, 0..=10)
    ties.add(Counted(1, 0, 3))
    ties.add(Counted(1, 0, 2))
    assert ties.at(4) == (cost: 4, used: 2)

@test
fn dnc_layers_match_the_quadratic_dp():
    for _ in 0..30:
        let n = rand(1, 12)
        let a = [rand(0, 9) for _ in 0..n]
        var pre = [0]
        for x in a: pre.push(pre.last! + x)
        let cost = (j, i) => (pre[i] - pre[j]) * (pre[i] - pre[j])
        var fast: List<?Int> = [if i == 0: 0 else: none for i in 0..=n]
        var slow = fast
        for _ in 0..rand(1, 4):
            fast = dnc_layer(fast, cost)
            var next: List<?Int> = List.filled(n + 1, none)
            for i in 1..=n:
                for j in 0..i:
                    if let v = slow[j]: next[i] min= v + cost(j, i)
            slow = next
            assert fast == slow

@test
fn knuth_matches_the_cubic_interval_dp():
    for _ in 0..30:
        let n = rand(1, 10)
        let a = [rand(1, 9) for _ in 0..n]
        var pre = [0]
        for x in a: pre.push(pre.last! + x)
        let dp = knuth(n, (l, r) => pre[r] - pre[l])
        var slow = [[0 for _ in 0..=n] for _ in 0..=n]
        for len in 2..=n:
            for i in 0..=n - len:
                let j = i + len
                slow[i][j] = pre[j] - pre[i] + min(slow[i][k] + slow[k][j] for k in i + 1..j)
        assert dp == slow

@test
fn aliens_finds_the_exact_count_optimum():
    // best sum of exactly k of the values: f(k) is convex (sorted, prefix sums)
    for _ in 0..50:
        let vals = sorted([rand(-10, 10) for _ in 0..rand(1, 8)])
        let k = rand(0, vals.len)
        let solve = lam => (
            val: sum(min(0, v + lam) for v in vals),
            count: sum(1 for v in vals if v + lam < 0),
        )
        assert aliens(k, -21..=21, solve) == sum(vals[i] for i in 0..k)

@test
fn subset_sums_are_every_sum_sorted():
    for _ in 0..50:
        let xs = [rand(-5, 9) for _ in 0..rand(0, 6)]
        var want: List<Int> = []
        for mask in 0..1 << xs.len:
            want.push(sum(xs[i] for i in 0..xs.len if (mask >> i) & 1 == 1))
        assert subset_sums(xs) == sorted(want)
