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
//! `convexity_break`). A sliding-window optimum is `a.windows(k).fold(op)` or `std.window.Window(op)`.
use std.algebra.{Monoid}
use std.math.{bit}
use std.seq.{first_true}

/// \(O(1)\) per evaluation.
/// The line `k * x + m`, the usual entry of a `LiChao`.
pub struct Line(pub k: Int, pub m: Int):
    /// Evaluate the line at `x`: `k * x + m`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `k * x + m` overflows `Int`
    /// example: `Line(2, 3).at(4) == 11`
    pub fn at(self, x: Int) -> Int = k * x + m

/// \(O(\log n)\) per `at`, \(O(\log n)\) per whole-line `add`, \(O(\log^2 n)\) on a segment.
/// The lower (`LiChao(min, over: 0..=1e9)`) or upper (`max`) envelope
/// of lines over the integer coordinates `over`. A line is anything with
/// `at(self, x: Int) -> K`, so a struct whose `at` returns `(cost, used)`
/// breaks ties by its second field with the same tree. Law: two lines cross
/// at most once on `xs`. `add` is \(O(\log n)\) for a whole line and \(O(\log^2 n)\)
/// on a segment (`on: lo..=hi`), `at` and `line_at` \(O(\log n)\), for n =
/// `xs.len`; memory is \(O(\mathit{lines})\) cells, allocated as lines arrive.
pub struct LiChao<L = Line, K = Int>(pick: (K, K) -> K, lo: Int, hi: Int):
    lines: List<?L>
    // a cell's children; 0 means no cell, since the root is no cell's child
    left: List<Int>
    right: List<Int>

    /// An empty envelope over the integer coordinates `over`, keeping at each `x` the value `pick`
    /// prefers (`min` for the lower envelope, `max` for the upper). No cell is allocated until a line
    /// arrives.
    ///
    /// `over` may be written half-open (`0..n`) or closed (`0..=hi`); the coordinates are its points.
    ///
    /// cost: \(O(1)\)
    /// empty: an empty `over` traps
    /// order: no sequence is returned; coordinates run in `over`'s order
    /// traps: when `over` is empty
    /// example: `LiChao(min, over: 0..=10).at(5) == None`
    pub fn new(pick: (K, K) -> K, over: Range<Int>) -> LiChao<L, K>:
        assert over.len > 0, "`LiChao` needs a non-empty `over`"
        LiChao(pick, over.start, over.start + over.len - 1, [None], [0], [0])

    /// Forget every line, keeping `pick` and `over`, so the tree can be reused (between test cases,
    /// say) without reallocating.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: never
    /// example: `var t = LiChao(min, over: 0..=10); t.add(Line(1, 0)); t.clear(); t.at(3) == None`
    pub fn clear(mut self):
        (lines, left, right) = ([None], [0], [0])

    /// The best value at `x` over every line covering `x`, as chosen by `pick`, or `None` when no line
    /// covers `x` (nothing added yet, or only segments that miss `x`).
    ///
    /// cost: \(O(\log n)\) for n = `over.len`
    /// ties: equal values are one value; which line produced it does not matter
    /// traps: when `x` is not in `over`
    /// example: `var t = LiChao(min, over: 0..=10); t.add(Line(1, 0)); t.add(Line(-1, 6)); t.at(1) == Some(1) and t.at(5) == Some(1)`
    pub fn at(self, x: Int) -> ?K:
        assert lo <= x and x <= hi
        var best: ?K = None
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

    /// The line attaining `at(x)`, or `None` when no line covers `x`.
    ///
    /// When several lines tie at `x` any of them may be returned; make `K` carry a tie-breaking second
    /// field (as with `at` returning `(cost, used)`) when the choice matters.
    ///
    /// cost: \(O(\log n)\) for n = `over.len`
    /// ties: unspecified among lines with equal values at `x`; break ties through `K`
    /// traps: when `x` is not in `over`
    /// example: `var t = LiChao(min, over: 0..=10); t.add(Line(1, 0)); t.add(Line(-1, 6)); t.line_at(8) == Some(Line(-1, 6))`
    pub fn line_at(self, x: Int) -> ?L:
        assert lo <= x and x <= hi
        var best: ?(line: L, val: K) = None
        var (i, l, r) = (0, lo, hi)
        while true:
            if let c = lines[i]:
                let v = c.at(x)
                if best == None or self.wins(v, best!.val): best = (line: c, val: v)
            let mid = l + (r - l) / 2
            if x <= mid:
                i = left[i]
                r = mid
            else:
                i = right[i]
                l = mid + 1
            if i == 0: break
        if let b = best: b.line else: None

    /// Add `line` on every coordinate of `over`, or only on the coordinates of `on`
    /// (`on: lo..=hi`). Later `at(x)` calls consider the line only where it was added.
    ///
    /// Law: any two lines added cross at most once on `over` (true for straight lines, and for `at`
    /// functions whose difference changes sign at most once).
    ///
    /// cost: \(O(\log n)\) for a whole line, \(O(\log^2 n)\) on a segment, for n = `over.len`; at most one new cell per level
    /// empty: an empty `on` adds nothing
    /// traps: when `on` reaches outside `over`
    /// example: `var t = LiChao(max, over: 0..=10); t.add(Line(0, 5), on: 2..=4); t.at(3) == Some(5) and t.at(5) == None`
    pub fn add(mut self, line: L, on: ?Range<Int> = None):
        if on == None:
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
        lines.push(None)
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
                if lines[i] == None:
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

/// For each row of the implicit `rows` x `cols` matrix `f`, the least `f(row, col)` and its
/// leftmost `col`, or `None` where `f` is `None` on the whole row.
///
/// Law: the leftmost argmin does not decrease from row to row (true when `f` is Monge;
/// `std.spec.monge_break` checks it). Rows are solved by divide and conquer on the middle row, so
/// `f` is called only inside the columns the law allows; it must be pure.
///
/// cost: \(O((\mathit{rows} + \mathit{cols}) \log \mathit{rows})\) calls of `f`, \(O(\mathit{rows})\) memory
/// empty: `rows == 0` gives `[]`; `cols == 0` gives `None` for every row
/// ties: the leftmost `col` among equal minima
/// order: one entry per row, row order
/// traps: when `rows < 0` or `cols < 0`
/// example: `monotone_minima(2, 3, (i, j) => Some((j - i) * (j - i))) == [Some((val: 0, at: 0)), Some((val: 0, at: 1))]`
pub fn monotone_minima(rows: Int, cols: Int, f: (Int, Int) -> ?Int) -> List<?(val: Int, at: Int)>:
    assert rows >= 0 and cols >= 0, "monotone_minima requires rows >= 0 and cols >= 0"
    var out: List<?(val: Int, at: Int)> = List.filled(rows, None)
    var todo = [(0, rows, 0, cols)]
    while let t = todo.pop():
        let (lo, hi, from, to) = t
        if lo >= hi: continue
        let mid = (lo + hi) / 2
        var best: ?(val: Int, at: Int) = None
        for j in from..to:
            if let v = f(mid, j):
                if best == None or v < best!.val: best = (val: v, at: j)
        out[mid] = best
        let cut = if let b = best: b.at else: from
        todo.push((lo, mid, from, if best == None: to else: cut + 1))
        todo.push((mid + 1, hi, cut, to))
    out

/// One layer of a partition DP: `next[i]` is the least `prev[j] + cost(j, i)` over `j < i` with
/// `prev[j]` present, or `None` when no such `j` exists, so `next[0]` is always `None`. Indices are
/// cuts `0..prev.len`, and `cost(j, i)` is the cost of the piece `[j, i)`.
///
/// Law: `cost` satisfies the quadrangle inequality (`std.spec.dnc_break` checks it), so the best
/// `j` moves right with `i`. `cost` is called only for `j < i` with `prev[j]` present.
///
/// cost: \(O(n \log n)\) calls of `cost` for n = `prev.len`
/// empty: `[]` gives `[]`
/// ties: only the value is kept; which `j` attains it does not matter
/// order: same length and index order as `prev`
/// traps: never
/// example: `dnc_layer([Some(0), None, None], (j, i) => (i - j) * (i - j)) == [None, Some(1), Some(4)]`
pub fn dnc_layer(prev: List<?Int>, cost: (Int, Int) -> Int) -> List<?Int>:
    let n = prev.len
    var next: List<?Int> = List.filled(n, None)
    var todo = [(1, n, 0, n)]
    unchecked:
        while let t = todo.pop():
            let (lo, hi, from, to) = t
            if lo >= hi: continue
            let mid = (lo + hi) / 2
            var best: ?Int = None
            var at = from
            for j in from..min(to, mid):
                if let p = prev[j]:
                    let v = p + cost(j, mid)
                    if best == None or v < best!:
                        best = v
                        at = j
            next[mid] = best
            todo.push((lo, mid, from, if best == None: to else: at + 1))
            todo.push((mid + 1, hi, at, to))
    next

/// The interval DP over cuts `0..=n`: `dp[i][i + 1] = 0` and `dp[i][j] = cost(i, j) + min over
/// i < k < j of dp[i][k] + dp[k][j]`, returned as the full `(n + 1) x (n + 1)` table. Entries with
/// `j <= i` are 0.
///
/// Law: `cost` is monotone under inclusion and satisfies the quadrangle inequality
/// (`std.spec.knuth_break` checks the split monotonicity it implies), so the best split of `[i, j)`
/// lies between those of `[i, j - 1)` and `[i + 1, j)`.
///
/// cost: \(O(n^2)\) time, memory and calls of `cost`
/// empty: `n == 0` gives `[[0]]`
/// ties: only the value is kept; which split attains it does not matter
/// order: `dp[i][j]` is row `i`, column `j`
/// traps: when `n < 0`
/// example: `knuth(2, (i, j) => j - i)[0][2] == 2`
/// example: `knuth(3, (i, j) => j - i)[0][3] == 5`
/// example: `knuth(1, (i, j) => j - i) == [[0, 0], [0, 0]]`
pub fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>>:
    assert n >= 0, "knuth requires n >= 0"
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

/// `f(k)` for a convex `f` known only through its penalized form: `solve(λ)` returns the least
/// `f(c) + λ * c` over every count `c`, and the least `c` attaining it.
///
/// The search finds the least `λ` whose count is at most `k` and returns `val - λ * k`, which is
/// exact because `f` is convex. Law: `f` is convex (`std.spec.convexity_break`), the counts fall as
/// `λ` grows, and `solve` reports the least optimal count. `L` is the value type (`Int`, `I128`).
/// Without `lambdas` the range is found by doubling out from `-1..=1`; pass it when `λ * k` could
/// overflow `L`.
///
/// cost: \(O(\log \mathit{lambdas}.\mathit{len})\) calls of `solve` (fewer once a count is exactly `k`); \(O(\log \lvert λ \cdot \rvert)\) more without `lambdas`
/// empty: an empty `lambdas` traps
/// order: `lambdas` is searched as an increasing range; one value is returned
/// ties: among counts with equal penalized value, `solve` must report the least; `aliens` relies on it
/// traps: when `k < 0`, when `lambdas` is empty, or when no `λ` in `lambdas` gives a count of at most `k`
/// example: `aliens(5, lam => (0..=10).map(c => (val: (c - 3) * (c - 3) + lam * c, count: c)).min(by: p => (p.val, p.count))!) == 4`
pub fn aliens<L: Add + Sub + Mul>(k: Int, solve: L -> (val: L, count: Int), lambdas: ?Range<L> = None) -> L:
    assert k >= 0
    var lo: L = -1
    var hi: L = 1
    if let r = lambdas:
        assert not r.is_empty
        lo = r.first!
        hi = r.last!
    else:
        while solve(hi).count > k: hi *= 2
        while solve(lo).count <= k: lo *= 2
    assert solve(hi).count <= k
    while lo < hi:
        let mid = lo + (hi - lo) / 2
        let at = solve(mid)
        if at.count == k: return at.val - mid * k
        if at.count < k: hi = mid
        else: lo = mid + 1
    solve(lo).val - lo * k

/// Every subset sum of `xs`, one per subset (repeats kept), sorted ascending: the half of a
/// meet-in-the-middle. Built by merging, not by sorting.
///
/// cost: \(O(2^n)\) time and memory for n = `xs.len`
/// empty: `[]` gives `[0]`
/// order: ascending; equal sums are indistinguishable
/// traps: when a sum overflows `Int`
/// example: `subset_sums([1, 2]) == [0, 1, 2, 3]`
/// example: `subset_sums([1, 1]) == [0, 1, 1, 2]`
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
    fn at(self, x: Int) -> (cost: Int, used: Int) = (cost: k * x + m, used:)

/// The shortest path from `nodes.first` to `nodes.last` along increasing edges
/// `nodes[j] -> nodes[i]`, `j < i`, of weight `cost(nodes[j], nodes[i])`: its total and its number
/// of edges, the shape `aliens` asks `solve` for
/// (`aliens(k, solve: pen => monge_path(cuts, cost: (j, i) => w(j, i) + pen))`).
///
/// Law: `cost` is Monge along `nodes`, `cost(a, c) + cost(b, d) <= cost(a, d) + cost(b, c)` for
/// `a <= b <= c <= d` (`std.spec.monge_break`). A single node is a path of no edges, total `0`.
///
/// cost: \(O(n)\) calls of `cost` (online LARSCH), \(O(n)\) memory, for n = `nodes.len`
/// empty: an empty `nodes` traps; one node gives `(val: 0, count: 0)`
/// ties: among paths of equal total the one found by LARSCH, not necessarily the fewest edges
/// traps: when `nodes` is empty
/// example: `monge_path([0, 1, 2], (a, b) => (b - a) * (b - a)) == (val: 2, count: 2)`
pub fn monge_path<N, K>(nodes: List<N>, cost: (N, N) -> K) -> (val: K, count: Int):
    let p = monge_paths(nodes, cost)
    var (i, count) = (nodes.len - 1, 0)
    while i > 0:
        i = p.prev[i]
        count += 1
    (val: p.val[nodes.len - 1], count:)

/// Every shortest path from `nodes.first`: `val[i]` is the least total weight to `nodes[i]` and
/// `prev[i]` the index before it on that path (`val[0] == 0`, `prev[0] == 0`). Same law as
/// `monge_path`; follow `prev` back from `i` to recover a path.
///
/// cost: \(O(n)\) calls of `cost` (online LARSCH), \(O(n)\) memory, for n = `nodes.len`
/// empty: an empty `nodes` traps
/// ties: among equal totals `prev[i]` is the predecessor found by LARSCH
/// order: both lists are indexed like `nodes`
/// traps: when `nodes` is empty
/// example: `monge_paths([0, 1, 2], (a, b) => (b - a) * (b - a)) == (val: [0, 1, 2], prev: [0, 0, 1])`
pub fn monge_paths<N, K>(nodes: List<N>, cost: (N, N) -> K) -> (val: List<K>, prev: List<Int>):
    assert nodes.len > 0
    var p = MongePath(nodes, cost)
    var prev = [0]
    while p.len < nodes.len: prev.push(p.next().prev)
    (val: [p.val(i) for i in 0..nodes.len], prev:)

/// \(O(1)\) per step; the next cut's optimum after \(O(1)\) amortized calls of `cost`.
/// `monge_path` one node at a time, for a DP whose next weight depends on the
/// optimum just found. `next()` returns the optimum at the next node and the
/// node before it; `val(i)` reads any node already reached.
pub struct MongePath<N, K: Add>(nodes: List<N>, cost: (N, N) -> K):
    // Online LARSCH: row r is node r + 1, column c is node c. Level l keeps a
    // stack of surviving columns for rows 2^(l+1) - 1 apart, and `bat[l]` is
    // the stack slot the best column for level l's next odd row starts at.
    vals: List<K> = []
    top: Int = -1
    sv: List<List<K>> = []
    sc: List<List<Int>> = []
    bv: List<K> = []
    bc: List<Int> = []
    bat: List<Int> = []

    fn w(self, j: Int, i: Int) -> K = cost(nodes[j], nodes[i])

    fn setup(mut self):
        if top >= 0: return
        assert nodes.len > 0
        vals = [K.zero]
        top = 0
        while (2 << top) <= nodes.len - 1: top += 1
        sv = [[] for _ in 0..top]
        sc = [[] for _ in 0..top]
        bv = [K.zero for _ in 0..top]
        bc = [0 for _ in 0..top]
        bat = [0 for _ in 0..top]

    /// Reach the next node: return the optimum at node `len` and the index of the node before it, then
    /// grow `len` by one. `cost` may read values already reached through `val`, so the weights can
    /// depend on earlier optima.
    ///
    /// cost: \(O(1)\) amortized calls of `cost`
    /// ties: among equal totals the predecessor found by LARSCH
    /// traps: when every node has been reached (`len == nodes.len`)
    /// example: `var p = MongePath([0, 1, 2], (a, b) => (b - a) * (b - a)); p.next(); p.next() == (val: 2, prev: 1)`
    pub fn next(mut self) -> (val: K, prev: Int):
        self.setup()
        assert vals.len < nodes.len, "`MongePath.next`: every node has been reached"
        let rows = nodes.len - 1
        let inp = vals.len - 1
        var l = 0
        var nv = K.zero
        var nc = 0
        while true:
            let r = inp >> l
            let col = if l == 0: inp else: sc[l - 1][r]
            if l == top:
                (nv, nc) = (if l == 0: (vals[col] + self.w(col, 0 + 1)) else: sv[l - 1][r], col)
                l -= 1
                break
            if r % 2 == 1:
                let row = ((r + 1) << l) - 1
                let pv = if l == 0: (vals[col] + self.w(col, row + 1)) else: sv[l - 1][r]
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
                let v = (vals[col] + self.w(col, (sv[l].len << (l + 1)) - 1 + 1))
                if not (v <= sv[l].last!): break
                let _ = sv[l].pop()
                let _ = sc[l].pop()
                (tv, tc, pushed) = (v, col, true)
            if pushed:
                sv[l].push(tv)
                sc[l].push(tc)
            else:
                let row = ((sv[l].len + 1) << (l + 1)) - 1
                if row < rows:
                    sv[l].push((vals[col] + self.w(col, row + 1)))
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
                let v = if l > 0 and idx == r: sv[l - 1][r] else: (vals[col] + self.w(col, row + 1))
                if not set or v <= nv:
                    (nv, nc, set) = (v, col, true)
                if col == end: break
                bat[l] += 1
            l -= 1
        vals.push(nv)
        (val: nv, prev: nc)
    /// The optimum at node `i`, which must already be reached; `val(0)` is 0.
    ///
    /// cost: \(O(1)\)
    /// traps: when `i < 0` or `i >= len`
    /// example: `var p = MongePath([0, 1, 2], (a, b) => (b - a) * (b - a)); p.next(); p.val(1) == 1`
    pub fn val(self, i: Int) -> K:
        assert 0 <= i and i < self.len
        if i == 0: K.zero else: vals[i]
    /// The number of nodes reached so far, `nodes.first` included, so a fresh `MongePath` has `len == 1`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `MongePath([0, 1, 2], (a, b) => (b - a) * (b - a)).len == 1`
    @get pub fn len(self) -> Int:
        if vals.len > 1: vals.len else: 1

fn smawk_rec(rows: List<Int>, cols: List<Int>, f: (Int, Int) -> Int, mut at: List<Int>):
    if rows.is_empty: return
    var kept: List<Int> = []
    for c in cols:
        while kept.len > 0:
            let r = rows[kept.len - 1]
            if f(r, kept.last!) <= f(r, c): break
            let _ = kept.pop()
        if kept.len < rows.len: kept.push(c)
    smawk_rec([rows[i] for i in 0..rows.len if i % 2 == 1], kept, f, mut at)
    var j = 0
    var i = 0
    while i < rows.len:
        let r = rows[i]
        let stop = if i + 1 < rows.len: at[rows[i + 1]] else: kept.last!
        var best = kept[j]
        var bv = f(r, best)
        while kept[j] != stop:
            j += 1
            let v = f(r, kept[j])
            if v < bv:
                bv = v
                best = kept[j]
        at[r] = best
        i += 2

/// For each row of the implicit `rows` x `cols` matrix `f`, the least `f(row, col)` and its
/// leftmost `col`.
///
/// Law: `f` is totally monotone (true when Monge; `std.spec.monge_break` checks that). Prefer
/// `monotone_minima` when `f` may be `None`: it needs only monotone argmins.
///
/// cost: \(O(\mathit{rows} + \mathit{cols})\) calls of `f`
/// empty: `rows == 0` gives `[]`; `cols == 0` traps (a row has no minimum)
/// ties: the leftmost `col` among equal minima
/// order: one entry per row, row order
/// traps: when `rows < 0`, or `cols <= 0` with `rows > 0`
/// example: `smawk(2, 3, (i, j) => (j - i) * (j - i)) == [(val: 0, at: 0), (val: 0, at: 1)]`
pub fn smawk(rows: Int, cols: Int, f: (Int, Int) -> Int) -> List<(val: Int, at: Int)>:
    assert rows >= 0, "`smawk` needs `rows >= 0`"
    if rows == 0: return []
    assert cols > 0, "`smawk` needs `cols > 0` when `rows > 0`"
    var at: List<Int> = List.filled(rows, 0)
    smawk_rec([i for i in 0..rows], [j for j in 0..cols], f, mut at)
    [(val: f(i, at[i]), at: at[i]) for i in 0..rows]

/// \(O(1)\) amortized per `add` and per `at_next`, \(O(\log n)\) per `at`.
/// The convex hull trick for lines added in slope order: falling slopes for
/// `pick: min`, rising for `max` (an out-of-order slope traps). The online
/// form of `LiChao` when slopes arrive sorted and no x-range is known.
/// Each line may carry a `tag` (a photo count, a parent index); `best_next` returns the winning
/// line's tag, and equal values go to the smaller tag, which is the tie rule `aliens` needs.
pub struct MonotoneHull<P>(pick: (Int, Int) -> Int = min):
    // Lines are stored negated under `max`, so the hull is always a lower envelope.
    ks: List<Int> = []
    ms: List<Int> = []
    tags: List<P> = []
    ptr: Int = 0
    pub last: ?Int = None

    fn sign(self) -> Int = if pick(0, 1) == 0: 1 else: -1

    fn val_at(self, i: Int, x: Int) -> Int = ks[i] * x + ms[i]

    fn covered(self, k: Int, m: Int) -> Bool:
        let (a, b) = (ks.len - 2, ks.len - 1)
        (m - ms[a]) * (ks[a] - ks[b]) <= (ms[b] - ms[a]) * (ks[a] - k)

    fn pop_line(mut self):
        let _ = ks.pop()
        let _ = ms.pop()
        let _ = tags.pop()
        if ptr >= ks.len and ptr > 0: ptr = max(ks.len - 1, 0)

    /// Add `line`, with `tag`. Slopes must arrive in order: non-increasing for `pick: min`,
    /// non-decreasing for `max`. A line of equal slope keeps only the better intercept, and lines that
    /// are never best are dropped.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: when the slope goes back
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.add(Line(-1, 3)); h.at(0) == Some(0) and h.at(2) == Some(1)`
    pub fn add(mut self, line: Line, tag: P = ()):
        let s = self.sign()
        let (k, m) = (s * line.k, s * line.m)
        let n = ks.len
        if n > 0:
            assert k <= ks[n - 1], "`MonotoneHull.add`: the slope went back"
            if k == ks[n - 1]:
                if m >= ms[n - 1]: return
                self.pop_line()
        while ks.len >= 2 and self.covered(k, m): self.pop_line()
        ks.push(k)
        ms.push(m)
        tags.push(tag)
    /// The best value at `x` with the tag of the line attaining it, for an `x` that never decreases
    /// between calls (a pointer walks the hull). `None` when empty.
    ///
    /// cost: \(O(1)\) amortized
    /// ties: equal values go to the smaller tag (the rule `aliens` needs)
    /// traps: when `x` is less than the previous `best_next` or `at_next` argument
    /// example: `var h = MonotoneHull<Int>(min); h.add(Line(1, 0), tag: 7); h.add(Line(-1, 2), tag: 3); h.best_next(1) == Some((val: 1, tag: 3))`
    pub fn best_next(mut self, x: Int) -> ?(val: Int, tag: P):
        assert last == None or x >= last!, "`best_next`/`at_next` need a non-decreasing `x`"
        last = Some(x)
        if ks.is_empty: return None
        while ptr + 1 < ks.len:
            let (a, b) = (self.val_at(ptr, x), self.val_at(ptr + 1, x))
            if b <= a: ptr += 1
            else: break
        Some((val: self.sign() * self.val_at(ptr, x), tag: tags[ptr]))
    /// The best value at any `x`, by binary search on the hull; `None` when empty.
    ///
    /// cost: \(O(\log n)\)
    /// ties: only the value is kept
    /// traps: never
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.add(Line(-1, 3)); h.at(5) == Some(-2)`
    pub fn at(self, x: Int) -> ?Int:
        if ks.is_empty: return None
        let lo = (0..ks.len - 1).first_true(mid => self.val_at(mid + 1, x) >= self.val_at(mid, x)) ?? (ks.len - 1)
        Some(self.sign() * self.val_at(lo, x))
    /// `at(x)` for an `x` that never decreases between calls; `None` when empty.
    ///
    /// cost: \(O(1)\) amortized
    /// ties: only the value is kept
    /// traps: when `x` is less than the previous `best_next` or `at_next` argument
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.add(Line(-1, 3)); h.at_next(0) == Some(0) and h.at_next(2) == Some(1)`
    pub fn at_next(mut self, x: Int) -> ?Int:
        if let b = self.best_next(x): Some(b.val) else: None
    /// The number of lines kept on the hull (dominated lines are not counted).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.add(Line(-1, 3)); h.len == 2`
    @get pub fn len(self) -> Int:
        ks.len
    /// Remove every line and reset the `at_next` pointer, keeping `pick`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.clear(); h.at(0) == None`
    pub fn clear(mut self):
        ks.clear()
        ms.clear()
        tags.clear()
        ptr = 0
        last = None

/// \(O(\log n)\) amortized per `add`, \(O(\log n)\) per `at`.
/// The convex hull trick with lines in any order and queries at any `x`,
/// no coordinate range needed (KACTL's line container). Use `LiChao` for
/// segment adds or non-line functions.
pub struct LineContainer(pick: (Int, Int) -> Int = min):
    // A max-hull treap keyed by slope; `min` stores negated lines. `ps[t]` is the last `x` at which
    // line `t` beats its successor, so queries descend by `ps` (KACTL's line container).
    ks: List<Int> = []
    ms: List<Int> = []
    ps: List<Int> = []
    pri: List<Int> = []
    ls: List<Int> = []
    rs: List<Int> = []
    root: Int = -1
    seed: Int = 1
    pub count: Int = 0

    fn sign(self) -> Int = if pick(0, 1) == 0: -1 else: 1

    fn floor_div(a: Int, b: Int) -> Int:
        let (n, d) = if b < 0: (-a, -b) else: (a, b)
        var q = n / d
        if q * d > n: q -= 1
        q

    fn meet(self, x: Int, y: Int) -> Int:
        if y < 0: return Int.MAX
        if ks[x] == ks[y]: return if ms[x] > ms[y]: Int.MAX else: Int.MIN
        LineContainer.floor_div(ms[y] - ms[x], ks[x] - ks[y])

    // (slopes < k, slopes >= k)
    fn split(mut self, t: Int, k: Int) -> (Int, Int):
        if t < 0: return (-1, -1)
        if ks[t] < k:
            let (a, b) = self.split(rs[t], k)
            rs[t] = a
            (t, b)
        else:
            let (a, b) = self.split(ls[t], k)
            ls[t] = b
            (a, t)

    fn merge(mut self, a: Int, b: Int) -> Int:
        if a < 0: return b
        if b < 0: return a
        if pri[a] > pri[b]:
            rs[a] = self.merge(rs[a], b)
            a
        else:
            ls[b] = self.merge(a, ls[b])
            b

    fn end(self, t: Int, right: Bool) -> Int:
        var u = t
        if u < 0: return -1
        while (if right: rs[u] else: ls[u]) >= 0: u = if right: rs[u] else: ls[u]
        u

    fn drop_end(mut self, t: Int, right: Bool) -> Int:
        if right:
            if rs[t] < 0:
                let c = ls[t]
                ls[t] = -1
                return c
            rs[t] = self.drop_end(rs[t], right)
        else:
            if ls[t] < 0:
                let c = rs[t]
                rs[t] = -1
                return c
            ls[t] = self.drop_end(ls[t], right)
        t

    fn best(self, x: Int) -> Int:
        var (t, at) = (root, -1)
        while t >= 0:
            if ps[t] >= x:
                at = t
                t = ls[t]
            else: t = rs[t]
        at
    /// Add `line`; slopes may arrive in any order. Lines that are never best are dropped.
    ///
    /// cost: \(O(\log n)\) amortized
    /// traps: never
    /// example: `var c = LineContainer(max); c.add(Line(1, 0)); c.add(Line(-1, 4)); c.at(0) == Some(4) and c.at(3) == Some(3)`
    pub fn add(mut self, line: Line):
        let sg = self.sign()
        seed = (seed * 1103515245 + 12345) % 2147483648
        ks.push(sg * line.k)
        ms.push(sg * line.m)
        ps.push(Int.MAX)
        pri.push(seed)
        ls.push(-1)
        rs.push(-1)
        let y = ks.len - 1
        let (lo, hi) = self.split(root, ks[y])
        var (eq, right) = self.split(hi, ks[y] + 1)
        var left = lo
        if eq >= 0:
            if ms[eq] >= ms[y]:
                root = self.merge(self.merge(left, eq), right)
                return
            count -= 1
        var z = self.end(right, false)
        ps[y] = self.meet(y, z)
        while z >= 0 and ps[y] >= ps[z]:
            right = self.drop_end(right, false)
            count -= 1
            z = self.end(right, false)
            ps[y] = self.meet(y, z)
        var keep = true
        var x = self.end(left, true)
        if x >= 0:
            ps[x] = self.meet(x, y)
            if ps[x] >= ps[y]:
                keep = false
                ps[x] = self.meet(x, z)
        let after = if keep: y else: z
        while x >= 0:
            left = self.drop_end(left, true)
            let w = self.end(left, true)
            if w < 0 or ps[w] < ps[x]:
                left = self.merge(left, x)
                break
            count -= 1
            ps[w] = self.meet(w, after)
            x = w
        if keep:
            count += 1
            left = self.merge(left, y)
        root = self.merge(left, right)
    /// The best value at `x`, or `None` when empty.
    ///
    /// cost: \(O(\log n)\)
    /// ties: only the value is kept
    /// traps: never
    /// example: `var c = LineContainer(min); c.add(Line(1, 0)); c.add(Line(-1, 4)); c.at(0) == Some(0) and c.at(3) == Some(1)`
    pub fn at(self, x: Int) -> ?Int:
        let t = self.best(x)
        if t < 0: None else: Some(self.sign() * (ks[t] * x + ms[t]))
    /// The line attaining `at(x)`, or `None` when empty.
    ///
    /// cost: \(O(\log n)\)
    /// ties: unspecified among lines with equal values at `x`
    /// traps: never
    /// example: `var c = LineContainer(max); c.add(Line(1, 0)); c.add(Line(-1, 4)); c.line_at(3) == Some(Line(1, 0))`
    pub fn line_at(self, x: Int) -> ?Line:
        let t = self.best(x)
        if t < 0: None else: Some(Line(self.sign() * ks[t], self.sign() * ms[t]))
    /// The number of lines kept (dominated lines are not counted).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var c = LineContainer(max); c.add(Line(1, 0)); c.add(Line(-1, 4)); c.len == 2`
    @get pub fn len(self) -> Int = count

/// The length of the longest increasing subsequence ending at each position, compared by `by`;
/// `strict: false` allows equal neighbours (non-decreasing). `xs.lis_ends().max()` is the LIS
/// length; `xs.lis()` in `std.seq` gives one.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory, one `by` call per element
/// empty: an empty array gives an empty array
/// order: same domain and positions as `self`
/// traps: never
/// example: `[3, 1, 4, 1, 5].lis_ends() == [1, 1, 2, 1, 3]`
/// example: `[1, 1, 2].lis_ends(strict: false) == [1, 2, 3]`
/// example: `[3, 1, 2].lis_ends(by: x => -x) == [1, 2, 2]`
pub fn lis_ends<D, T, K>(self: [D]T, strict: Bool = true, by: T -> K = x => x) -> [D]Int:
    var tails: List<K> = []
    var out: [D]Int = 0
    for i in D:
        let k = by(self[i])
        let lo = (0..tails.len).first_true(mid => not (tails[mid] < k or (not strict and tails[mid] == k))) ?? tails.len
        if lo == tails.len: tails.push(k)
        else: tails[lo] = k
        out[i] = lo + 1
    out

/// Every mask of `bits` bits containing `m`, increasing from `m` to `2^bits - 1`.
///
/// cost: \(O(2^(\mathit{bits} - \mathit{popcount}(m)))\)
/// empty: never empty; `m` itself comes first
/// order: increasing
/// traps: when `bits < 0` or `m` is not in `0..2^bits`
/// example: `supermasks(0b01, 2).collect() == [1, 3]`
pub fn supermasks(m: Int, bits: Int) -> yields Int:
    assert bits >= 0 and 0 <= m and m < (1 << bits), "`m` must be a mask of `bits` bits"
    let full = (1 << bits) - 1
    var s = m
    while true:
        yield s
        if s == full: break
        s = (s + 1) | m

/// Every mask of `bits` bits with exactly `k` bits set, increasing (Gosper's hack).
///
/// cost: \(O(C(\mathit{bits}, k))\)
/// empty: nothing when `k > bits`; `k == 0` yields only `0`
/// order: increasing
/// traps: when `bits < 0` or `k < 0`
/// example: `masks_of_size(3, 2).collect() == [3, 5, 6]`
pub fn masks_of_size(bits: Int, k: Int) -> yields Int:
    assert bits >= 0 and k >= 0, "`bits` and `k` must be non-negative"
    if k > bits: return
    if k == 0:
        yield 0
        return
    let end = 1 << bits
    var x = (1 << k) - 1
    while x < end:
        yield x
        let c = x & -x
        let r = x + c
        x = (((r ^ x) >> 2) / c) | r

/// `out[mask]` is the fold under `op` of `xs[i]` for `i` in `mask`, in increasing `i`;
/// `out[0]` is `op`'s identity (W10).
///
/// cost: \(O(2^n)\) time and memory, one `op` call per non-empty mask, for n = `xs.len`
/// empty: `[]` gives `[identity]`
/// order: indexed by mask; within a mask, elements fold in increasing index
/// traps: never
/// example: `subset_folds([1, 2, 4], +) == [0, 1, 2, 3, 4, 5, 6, 7]`
pub fn subset_folds<T, M>(xs: List<T>, op: M) -> List<T>:
    let m = Monoid<T>.of(op)
    let n = xs.len
    var out: List<T> = List.filled(1 << n, m.id)
    var top = 0
    for mask in 1..(1 << n):
        if mask == 1 << (top + 1): top += 1
        out[mask] = m.op(out[mask ^ (1 << top)], xs[top])
    out

/// Depth-first backtracking from `start`, where `next(s)` yields the states one step on. Every
/// state reached is yielded as a `Step` with `.state` and `.depth` (`start` at depth 0), in
/// preorder: a state before its children, children in the order `next` yields them.
///
/// Calling `.prune()` on the current step skips every state below it; stop the whole search with
/// `break` or `return` in the `for` body (decision 126). States are not deduplicated.
///
/// cost: \(O(\mathit{visited})\) steps plus the cost of `next`
/// order: preorder; children in `next` order
/// traps: never
/// example: `backtrack(1, x => if x < 4: [2 * x, 2 * x + 1] else: []).map(s => s.state).collect() == [1, 2, 4, 5, 3, 6, 7]`
/// example: `backtrack(1, x => if x < 4: [2 * x, 2 * x + 1] else: []).map(s => s.depth).collect() == [0, 1, 2, 2, 1, 2, 2]`
pub fn backtrack<S>(start: S, next: S -> yields S) -> yields Step<S>:
    var stack: List<(S, Int)> = [(start, 0)]
    while let (s, d) = stack.pop():
        yield Step(s, d)
        let kids: List<S> = [c for c in next(s)]
        var i = kids.len - 1
        while i >= 0:
            stack.push((kids[i], d + 1))
            i -= 1

/// One state reached by `backtrack`.
pub struct Step<S>(pub state: S, pub depth: Int):
    /// Skip every state below this one: `backtrack` does not call `next` on it. Has effect only on
    /// the step just yielded, before the `for` body continues.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: with `next` as `x => if x < 4: [2 * x, 2 * x + 1] else: []`, a `for s in backtrack(1, next)` body that calls `s.prune()` when `s.state == 2` sees the states `[1, 2, 3, 6, 7]`
    pub fn prune(self) = todo

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
                lines.push((l:, a: lo, b: hi))
            else:
                let (a, b) = (rand(lo - 3, hi), rand(lo, hi + 3))
                low.add(l, on: a..=b)
                high.add(l, on: a..=b)
                lines.push((l:, a:, b:))
            for x in lo..=hi:
                let here = [e.l.at(x) for e in lines if e.a <= x and x <= e.b]
                assert low.at(x) == here.min()!
                assert high.at(x) == here.max()!
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
        var fast: List<?Int> = [if i == 0: 0 else: None for i in 0..=n]
        var slow = fast.copy()
        for _ in 0..rand(1, 4):
            fast = dnc_layer(fast, cost)
            var next: List<?Int> = List.filled(n + 1, None)
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
                slow[i][j] = pre[j] - pre[i] + [slow[i][k] + slow[k][j] for k in i + 1..j].min()!
        assert dp == slow

@test
fn aliens_finds_the_exact_count_optimum():
    let (lo, hi) = (-21, 21)
    for _ in 0..50:
        let vals = [rand(-10, 10) for _ in 0..rand(1, 8)].sorted()
        let k = rand(0, vals.len)
        fn solve(lam: Int) -> (val: Int, count: Int):
            var (v, c) = (0, 0)
            for x in vals:
                if x + lam < 0:
                    v += x + lam
                    c += 1
            (val: v, count: c)
        let want = sum(vals[i] for i in 0..k)
        assert aliens(k, solve, lambdas: lo..=hi) == want

@test
fn aliens_searches_a_wide_i128_slope_range():
    let big = I128(1_000_000_000_000) * I128(1_000_000_000_000)
    for _ in 0..20:
        let vals = [big * I128(rand(-10, 10)) for _ in 0..rand(1, 8)].sorted()
        let k = rand(0, vals.len)
        fn solve2(lam: I128) -> (val: I128, count: Int):
            var (v, c) = (I128(0), 0)
            for x in vals:
                if x + lam < I128(0):
                    v += x + lam
                    c += 1
            (val: v, count: c)
        var want = I128(0)
        for i in 0..k: want += vals[i]
        assert aliens(k, solve2, lambdas: -big * I128(21)..=big * I128(21)) == want

@test
fn subset_sums_are_every_sum_sorted():
    for _ in 0..50:
        let xs = [rand(-5, 9) for _ in 0..rand(0, 6)]
        var want: List<Int> = [sum(xs[i] for i in 0..xs.len if mask.bit(i)) for mask in 0..1 << xs.len]
        assert subset_sums(xs) == want.sorted()

@test
fn monge_paths_match_the_quadratic_dp():
    for _ in 0..200:
        let n = rand(1, 30)
        var p = [0]
        for _ in 1..n: p.push(p.last! + rand(0, 9))
        let c = rand(-20, 20)
        fn w(j: Int, i: Int) -> Int = (p[i] - p[j]) * (i - j) + c
        let got = monge_paths([i for i in 0..n], w)
        var best = [0]
        for i in 1..n:
            var b = best[0] + w(0, i)
            for j in 1..i: b = min(b, best[j] + w(j, i))
            best.push(b)
        assert got.val == best
        for i in 1..n: assert best[got.prev[i]] + w(got.prev[i], i) == best[i]
