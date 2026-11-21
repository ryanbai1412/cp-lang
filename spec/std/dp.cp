//! The DP toolkit: the structures and optimizations a DP calls by name.
//!
//! A minimum or maximum of lines (the convex hull trick) is a `LiChao`; a
//! layer whose optimal split moves right is `dnc_layer` (over `monotone_minima`); an interval DP with
//! monotone splits is `knuth`; "exactly `k` pieces" over a convex cost is
//! `aliens`. Meet in the middle starts from `subset_sums`; sums over subsets and
//! supersets of masks are `std.ntt`'s `sos(mut f, over: Sub | Sup = Sub, inverse:)`. The
//! recurrence itself stays in your code, and a memoized recursion is
//! `@memo`. Each optimization names the law it needs; `std.spec` checks it
//! on small inputs (`monge_break`, `dnc_break`, `knuth_break`,
//! `convexity_break`). A sliding-window optimum is `std.window.MonoDeque`.

/// O(1) per evaluation.
/// The line `k * x + m`, the usual entry of a `LiChao`.
pub struct Line(k: Int, m: Int):
/// O(1).
    /// Evaluate the line `k * x + m` at `x`. O(1).
    pub fn at(self, x: Int) -> Int = k * x + m

/// O(log n) per `at`, O(log n) per whole-line `add`, O(log^2 n) on a segment.
/// The lower (`LiChao(min, over: 0..=1e9)`) or upper (`max`) envelope
/// of lines over the integer coordinates `over`. A line is anything with
/// `at(self, x: Int) -> K`, so a struct whose `at` returns `(cost, used)`
/// breaks ties by its second field with the same tree. Law: two lines cross
/// at most once on `xs`. `add` is O(log n) for a whole line and O(log^2 n)
/// on a segment (`on: lo..=hi`), `at` and `line_at` O(log n), for n =
/// `xs.len`; memory is O(lines) cells, allocated as lines arrive.
pub struct LiChao<L = Line, K = Int>(pick: (K, K) -> K, lo: Int, hi: Int):
    lines: List<?L>
    // a cell's children; 0 is none, since the root is no cell's child
    left: List<Int>
    right: List<Int>

/// O(1).
    /// Create a Li Chao tree over integer x-domain `over` with comparator `pick`. O(1).
    pub fn new(pick: (K, K) -> K, over: Range<Int>) -> LiChao<L, K>:
        let xs = over
        assert not xs.is_empty
        LiChao(pick, xs.first!, xs.last!, [none], [0], [0])

/// O(1) amortized.
    /// Forget every line. O(1) amortized.
    pub fn clear(mut self):
        (lines, left, right) = ([none], [0], [0])

/// O(log n).
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

/// O(log n).
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

/// O(log n), O(log^2 n) on a segment.
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

/// O((rows + cols) log rows) calls of `f`.
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

/// O(n log n) calls of `cost`.
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

/// O(n^2) time and memory.
/// The interval DP over cuts `0..=n`: `dp[i][i + 1] = 0` and `dp[i][j] =
/// cost(i, j) + min over i < k < j of dp[i][k] + dp[k][j]`. Law: `cost` is
/// monotone under inclusion and satisfies the quadrangle inequality
/// (`std.spec.knuth_break` checks the split monotonicity it implies), so the
/// best split of `[i, j)` lies between those of `[i, j - 1)` and `[i + 1, j)`.
/// O(n^2) time and memory.
pub fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>>:
    var dp: List<List<Int>> = []
    var opt: List<List<0..2_147_483_648>> = []
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

/// O(log lambdas.len) calls of `solve`, fewer once a count is exactly `k`.
/// `f(k)` for a convex `f` known only through its penalized form:
/// `solve(λ)` is the least `f(c) + λ * c` over every count `c`, and the least
/// `c` attaining it. Law: `f` is convex (`std.spec.convexity_break`), and the
/// first `λ` of `lambdas` gives a count of at most `k` somewhere in the range
/// (the counts fall as `λ` grows). `L` is the value type (`Int`, `I128`).
/// Without `lambdas` the range is found by doubling out from `-1..=1`
/// (O(log |λ*|) more calls); pass it when `λ * k` could overflow `L`.
pub fn aliens<L>(k: Int, solve: L -> (val: L, count: Int), lambdas: ?Range<L> = none) -> L:
    let r = lambdas ?? penalties(k, solve)
    var lo = r.first!
    var hi = r.last!
    while lo < hi:
        let mid = (lo + (hi - lo) / 2)!
        let at = solve(mid)
        if at.count == k: return at.val - mid * k
        if at.count < k: hi = mid else: lo = (mid + 1)!
    solve(lo).val - lo * k

// The penalties around the answer: the counts fall as the penalty grows, so
// double until the count is at most `k` above and at least `k` below.
fn penalties<L>(k: Int, solve: L -> (val: L, count: Int)) -> Range<L>:
    var hi: L = 1
    while solve(hi).count > k: hi = hi * 2
    var lo: L = -1
    while solve(lo).count < k: lo = lo * 2
    lo..=hi

/// O(2^n) time and memory.
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

// Online LARSCH over nodes 0..n; the engine of `monge_path` and `monge_paths`.
fn larsch<K>(n: Int, w: (Int, Int) -> K) -> (val: List<K>, prev: List<Int>):
    // Online LARSCH: row r is node r + 1, column c is node c. Level l keeps a
    // stack of surviving columns for rows 2^(l+1) - 1 apart, and `bat[l]` is
    // the stack slot the best column for level l's next odd row starts at.
    var val = List.filled(max(n, 1), K.zero)
    var prev = List.filled(max(n, 1), 0)
    let rows = n - 1
    if rows <= 0: return (val: val, prev: prev)
    var top = 0
    while (2 << top) <= rows: top += 1
    var sv: List<List<K>> = [[] for _ in 0..top]
    var sc: List<List<Int>> = [[] for _ in 0..top]
    for l in 0..top:
        sv[l].reserve((rows >> (l + 1)) + 1)
        sc[l].reserve((rows >> (l + 1)) + 1)
    var bv: List<K> = [K.zero for _ in 0..top]
    var bc = [0 for _ in 0..top]
    var bat = [0 for _ in 0..top]
    fn get(row: Int, col: Int) -> K = val[col] + w(col, row + 1)
    for inp in 0..rows:
        var l = 0
        var nv = K.zero
        var nc = 0
        while true:
            let r = inp >> l
            let col = if l == 0: inp else: sc[l - 1][r]
            if l == top:
                (nv, nc) = (if l == 0: get(0, col) else: sv[l - 1][r], col)
                l -= 1
                break
            if r % 2 == 1:
                let row = ((r + 1) << l) - 1
                let pv = if l == 0: get(row, col) else: sv[l - 1][r]
                if bat[l] == r or pv <= bv[l]:
                    (bv[l], bc[l], bat[l]) = (pv, col, r)
            let keep = (r + 1) / 2
            if bat[l] == r:
                sv[l].truncate(keep)
                sc[l].truncate(keep)
                if l + 1 < top: bat[l + 1] = keep
            var pushed = false
            var (tv, tc) = (K.zero, 0)
            while sv[l].len > keep:
                let v = get((sv[l].len << (l + 1)) - 1, col)
                if not (v <= sv[l].last!): break
                sv[l].pop()
                sc[l].pop()
                (tv, tc, pushed) = (v, col, true)
            if pushed:
                sv[l].push(tv)
                sc[l].push(tc)
            else:
                let row = ((sv[l].len + 1) << (l + 1)) - 1
                if row < rows:
                    sv[l].push(get(row, col))
                    sc[l].push(col)
            if r % 2 == 1:
                (nv, nc) = (bv[l], bc[l])
                l -= 1
                break
            if ((r + 2) << l) - 1 >= rows:
                nc = col
                break
            l += 1
        while l >= 0:
            let r = inp >> l
            let row = ((r + 1) << l) - 1
            let end = nc
            (bv[l], bc[l]) = (nv, end)
            var set = false
            while true:
                let idx = bat[l]
                let col = if l == 0: idx else: sc[l - 1][idx]
                let v = if l > 0 and idx == r: sv[l - 1][r] else: get(row, col)
                if not set or v <= nv:
                    (nv, nc, set) = (v, col, true)
                if col == end: break
                bat[l] += 1
            l -= 1
        (val[inp + 1], prev[inp + 1]) = (nv, nc)
    (val: val, prev: prev)

/// O(n) calls of `cost` (online LARSCH), O(n) memory, for n = `nodes.len`.
/// The shortest path from `nodes.first` to `nodes.last` along increasing
/// edges `nodes[j] -> nodes[i]`, `j < i`, of weight `cost(nodes[j], nodes[i])`:
/// its total and its number of edges, the shape `aliens` asks `solve` for
/// (`aliens(k, solve: pen => monge_path(cuts, cost: (j, i) => w(j, i) + pen))`).
/// Law: `cost` is Monge along `nodes`, `cost(a, c) + cost(b, d) <= cost(a, d) +
/// cost(b, c)` for `a <= b <= c <= d` (`std.spec.monge_break`); on a tie the
/// fewest edges is not guaranteed.
pub fn monge_path<N, K>(nodes: List<N>, cost: (N, N) -> K) -> (val: K, count: Int):
    let t = monge_paths(nodes, cost: cost)
    var count = 0
    var at = nodes.len - 1
    while at > 0:
        at = t.prev[at]
        count += 1
    (val: t.val[nodes.len - 1], count: count)

/// O(n) calls of `cost` (online LARSCH), O(n) memory, for n = `nodes.len`.
/// Every shortest path from `nodes.first`: `val[i]` is the least total weight
/// to `nodes[i]` and `prev[i]` the index before it on that path (`prev[0] ==
/// 0`). Same law as `monge_path`.
pub fn monge_paths<N, K>(nodes: List<N>, cost: (N, N) -> K) -> (val: List<K>, prev: List<Int>) =
    larsch(nodes.len, (j, i) => cost(nodes[j], nodes[i]))

/// O(1) per step; the next cut's optimum after O(1) amortized calls of `cost`.
/// `monge_path` one node at a time, for a DP whose next weight depends on the
/// optimum just found. `next()` returns the optimum at the next node and the
/// node before it; `val(i)` reads any node already reached.
pub struct MongePath<N, K>(nodes: List<N>, cost: (N, N) -> K):
    /// O(1).
    pub fn next(mut self) -> (val: K, prev: Int) = todo
    /// O(1); the optimum at node `i`, reached already.
    pub fn val(self, i: Int) -> K = todo
    /// O(1); the nodes reached so far, `nodes.first` included.
    pub @get fn len(self) -> Int = todo

/// O(rows + cols) calls of `f` (SMAWK). For each row, the least `f(row, col)`
/// and its leftmost `col`. Law: `f` is totally monotone (true when Monge;
/// `std.spec.monge_break` checks that). Prefer `monotone_minima` when `f` may
/// be `none`: it needs only monotone argmins.
pub fn smawk(rows: Int, cols: Int, f: (Int, Int) -> Int) -> List<(val: Int, at: Int)> = todo

/// Which arguments of a min-plus or max-plus convolution have the shape that
/// makes it fast: both, only the right one, or neither.
pub enum Shape = Both | Right | Neither

/// O(n + m) for `Both`, O((n + m) log n) for `Right`, O(n m) for `Neither`.
/// `c[k]` is the least `a[i] + b[j]` over `i + j = k`; `convex` says which
/// arguments are convex (`Both` merges the slopes; `Right` runs
/// `monotone_minima`).
pub fn min_plus_convolve(a: List<Int>, b: List<Int>, convex: Shape = Both) -> List<Int> = todo

/// O(n + m) for `Both`, O((n + m) log n) for `Right`, O(n m) for `Neither`.
/// `c[k]` is the largest `a[i] + b[j]` over `i + j = k`; `concave` says
/// which arguments are concave.
pub fn max_plus_convolve(a: List<Int>, b: List<Int>, concave: Shape = Both) -> List<Int> = todo

/// O(1) amortized per `add` and per `at_next`, O(log n) per `at`.
/// The convex hull trick for lines added in slope order: falling slopes for
/// `pick: min`, rising for `max` (an out-of-order slope traps). The online
/// form of `LiChao` when slopes arrive sorted and no x-range is known.
pub struct MonotoneHull(pick: (Int, Int) -> Int = min):
    /// O(1) amortized; the slope must not go back.
    pub fn add(mut self, line: Line) = todo
    /// O(log n); the best value at `x`, `none` when empty.
    pub fn at(self, x: Int) -> ?Int = todo
    /// O(1) amortized; `at(x)` for an `x` that never decreases between calls.
    pub fn at_next(mut self, x: Int) -> ?Int = todo
    /// O(1).
    pub @get fn len(self) -> Int = todo
    /// O(1).
    pub fn clear(mut self) = todo

/// O(log n) amortized per `add`, O(log n) per `at`.
/// The convex hull trick with lines in any order and queries at any `x`,
/// no coordinate range needed (KACTL's line container). Use `LiChao` for
/// segment adds or non-line functions.
pub struct LineContainer(pick: (Int, Int) -> Int = min):
    /// O(log n) amortized.
    pub fn add(mut self, line: Line) = todo
    /// O(log n); the best value at `x`, `none` when empty.
    pub fn at(self, x: Int) -> ?Int = todo
    /// O(log n); the line that is best at `x`.
    pub fn line_at(self, x: Int) -> ?Line = todo
    /// O(1).
    pub @get fn len(self) -> Int = todo

/// O(n log n). The positions of one longest increasing subsequence of `xs`
/// by `by`; `strict: false` allows equal neighbours.
pub fn lis<D, T, K>(xs: [D]T, strict: Bool = true, by: T -> K = x => x) -> List<D> = todo

/// O(n log n). The length of the longest increasing subsequence ending at
/// each position (`lis_ends(xs).max()` is the LIS length).
pub fn lis_ends<D, T, K>(xs: [D]T, strict: Bool = true, by: T -> K = x => x) -> [D]Int = todo

/// O(2^popcount(m)). Every submask of `m`, from `m` down to 0.
pub fn submasks(m: Int) -> yields Int = todo

/// O(2^(bits - popcount(m))). Every mask of `bits` bits containing `m`,
/// from `m` up.
pub fn supermasks(m: Int, bits: Int) -> yields Int = todo

/// O(C(bits, k)). Every mask of `bits` bits with exactly `k` set, increasing
/// (Gosper's hack).
pub fn masks_of_size(bits: Int, k: Int) -> yields Int = todo

/// O(2^n) time and memory. `out[mask]` is the fold under `op` of `xs[i]`
/// for `i` in `mask`; `out[0]` is `op`'s identity (W10).
pub fn subset_folds<T, M>(xs: List<T>, op: M) -> List<T> = todo

/// O(visited). Depth-first backtracking from `start`, where `next(s)` yields
/// the states one step on. Each `Step` has `.state`, `.depth` and `.prune()`,
/// which skips the subtree below it; stop with `break` or `return` in the
/// `for` body (decision 126).
pub fn backtrack<S>(start: S, next: S -> yields S) -> yields Step<S> = todo

/// One state reached by `backtrack`.
pub struct Step<S>(state: S, depth: Int):
    /// O(1); `backtrack` skips every state below this one.
    pub fn prune(self) = todo

@test
fn li_chao_matches_a_scan_over_lines_and_segments():
    for _ in 0..100:
        let (lo, hi) = (rand(-20, 0), rand(1, 20))
        var low = LiChao(min, over: lo..=hi)
        var high = LiChao(max, over: lo..=hi)
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
    var ties = LiChao<Counted, (cost: Int, used: Int)>(min, over: 0..=10)
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
        assert aliens(k, solve, lambdas: -21..=21) == sum(vals[i] for i in 0..k)

@test
fn aliens_searches_a_wide_i128_slope_range():
    let big = I128(1_000_000_000_000) * I128(1_000_000_000_000)
    for _ in 0..20:
        let vals = sorted([big * rand(-10, 10) for _ in 0..rand(1, 8)])
        let k = rand(0, vals.len)
        fn solve(lam: I128) -> (val: I128, count: Int) = (
            val: sum(min(I128(0), v + lam) for v in vals),
            count: sum(1 for v in vals if v + lam < 0),
        )
        assert aliens(k, solve, lambdas: -big * 21..=big * 21) == sum(vals[i] for i in 0..k)

@test
fn subset_sums_are_every_sum_sorted():
    for _ in 0..50:
        let xs = [rand(-5, 9) for _ in 0..rand(0, 6)]
        var want: List<Int> = []
        for mask in 0..1 << xs.len:
            want.push(sum(xs[i] for i in 0..xs.len if (mask >> i) & 1 == 1))
        assert subset_sums(xs) == sorted(want)

@test
fn monge_paths_match_the_quadratic_dp():
    for _ in 0..200:
        let n = rand(1, 30)
        var p = [0]
        for _ in 1..n: p.push(p.last! + rand(0, 9))
        let c = rand(-20, 20)
        fn w(j: Int, i: Int) -> Int = (p[i] - p[j]) * (i - j) + c
        let got = larsch(n, w)
        var best = [0]
        for i in 1..n: best.push(min(best[j] + w(j, i) for j in 0..i)!)
        assert got.val == best
        for i in 1..n: assert best[got.prev[i]] + w(got.prev[i], i) == best[i]
