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

/// O(1) per evaluation.
/// The line `k * x + m`, the usual entry of a `LiChao`.
pub struct Line(k: Int, m: Int):
    /// Evaluate the line at `x`: `k * x + m`.
    ///
    /// cost: O(1)
    /// traps: when `k * x + m` overflows `Int`
    /// example: `Line(2, 3).at(4) == 11`
    pub fn at(self, x: Int) -> Int = todo

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
    // a cell's children; 0 means no cell, since the root is no cell's child
    left: List<Int>
    right: List<Int>

    /// An empty envelope over the integer coordinates `over`, keeping at each `x` the value `pick`
    /// prefers (`min` for the lower envelope, `max` for the upper). No cell is allocated until a line
    /// arrives.
    ///
    /// `over` may be written half-open (`0..n`) or closed (`0..=hi`); the coordinates are its points.
    ///
    /// cost: O(1)
    /// empty: an empty `over` traps
    /// order: no sequence is returned; coordinates run in `over`'s order
    /// traps: when `over` is empty
    /// example: `LiChao(min, over: 0..=10).at(5) == None`
    pub fn new(pick: (K, K) -> K, over: Range<Int>) -> LiChao<L, K> = todo

    /// Forget every line, keeping `pick` and `over`, so the tree can be reused (between test cases,
    /// say) without reallocating.
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `var t = LiChao(min, over: 0..=10); t.add(Line(1, 0)); t.clear(); t.at(3) == None`
    pub fn clear(mut self) = todo

    /// The best value at `x` over every line covering `x`, as chosen by `pick`, or `None` when no line
    /// covers `x` (nothing added yet, or only segments that miss `x`).
    ///
    /// cost: O(log n) for n = `over.len`
    /// ties: equal values are one value; which line produced it does not matter
    /// traps: when `x` is not in `over`
    /// example: `var t = LiChao(min, over: 0..=10); t.add(Line(1, 0)); t.add(Line(-1, 6)); t.at(1) == Some(1) and t.at(5) == Some(1)`
    pub fn at(self, x: Int) -> ?K = todo

    /// The line attaining `at(x)`, or `None` when no line covers `x`.
    ///
    /// When several lines tie at `x` any of them may be returned; make `K` carry a tie-breaking second
    /// field (as with `at` returning `(cost, used)`) when the choice matters.
    ///
    /// cost: O(log n) for n = `over.len`
    /// ties: unspecified among lines with equal values at `x`; break ties through `K`
    /// traps: when `x` is not in `over`
    /// example: `var t = LiChao(min, over: 0..=10); t.add(Line(1, 0)); t.add(Line(-1, 6)); t.line_at(8) == Some(Line(-1, 6))`
    pub fn line_at(self, x: Int) -> ?L = todo

    /// Add `line` on every coordinate of `over`, or only on the coordinates of `on`
    /// (`on: lo..=hi`). Later `at(x)` calls consider the line only where it was added.
    ///
    /// Law: any two lines added cross at most once on `over` (true for straight lines, and for `at`
    /// functions whose difference changes sign at most once).
    ///
    /// cost: O(log n) for a whole line, O(log^2 n) on a segment, for n = `over.len`; at most one new cell per level
    /// empty: an empty `on` adds nothing
    /// traps: when `on` reaches outside `over`
    /// example: `var t = LiChao(max, over: 0..=10); t.add(Line(0, 5), on: 2..=4); t.at(3) == Some(5) and t.at(5) == None`
    pub fn add(mut self, line: L, on: ?Range<Int> = None) = todo

    fn wins(self, a: K, b: K) -> Bool = todo

    fn kid(mut self, i: Int, right_side: Bool) -> Int = todo

    // Keep the line better at the middle of `l..=r` here; the other one is
    // better on at most one half, and goes down into it.
    fn settle(mut self, at: Int, lo: Int, hi: Int, line: L) = todo

/// For each row of the implicit `rows` x `cols` matrix `f`, the least `f(row, col)` and its
/// leftmost `col`, or `None` where `f` is `None` on the whole row.
///
/// Law: the leftmost argmin does not decrease from row to row (true when `f` is Monge;
/// `std.spec.monge_break` checks it). Rows are solved by divide and conquer on the middle row, so
/// `f` is called only inside the columns the law allows; it must be pure.
///
/// cost: O((rows + cols) log rows) calls of `f`, O(rows) memory
/// empty: `rows == 0` gives `[]`; `cols == 0` gives `None` for every row
/// ties: the leftmost `col` among equal minima
/// order: one entry per row, row order
/// traps: when `rows < 0` or `cols < 0`
/// example: `monotone_minima(2, 3, (i, j) => Some((j - i) * (j - i))) == [Some((val: 0, at: 0)), Some((val: 0, at: 1))]`
pub fn monotone_minima(rows: Int, cols: Int, f: (Int, Int) -> ?Int) -> List<?(val: Int, at: Int)> = todo

/// One layer of a partition DP: `next[i]` is the least `prev[j] + cost(j, i)` over `j < i` with
/// `prev[j]` present, or `None` when no such `j` exists, so `next[0]` is always `None`. Indices are
/// cuts `0..prev.len`, and `cost(j, i)` is the cost of the piece `[j, i)`.
///
/// Law: `cost` satisfies the quadrangle inequality (`std.spec.dnc_break` checks it), so the best
/// `j` moves right with `i`. `cost` is called only for `j < i` with `prev[j]` present.
///
/// cost: O(n log n) calls of `cost` for n = `prev.len`
/// empty: `[]` gives `[]`
/// ties: only the value is kept; which `j` attains it does not matter
/// order: same length and index order as `prev`
/// traps: never
/// example: `dnc_layer([Some(0), None, None], (j, i) => (i - j) * (i - j)) == [None, Some(1), Some(4)]`
pub fn dnc_layer(prev: List<?Int>, cost: (Int, Int) -> Int) -> List<?Int> = todo

/// The interval DP over cuts `0..=n`: `dp[i][i + 1] = 0` and `dp[i][j] = cost(i, j) + min over
/// i < k < j of dp[i][k] + dp[k][j]`, returned as the full `(n + 1) x (n + 1)` table. Entries with
/// `j <= i` are 0.
///
/// Law: `cost` is monotone under inclusion and satisfies the quadrangle inequality
/// (`std.spec.knuth_break` checks the split monotonicity it implies), so the best split of `[i, j)`
/// lies between those of `[i, j - 1)` and `[i + 1, j)`.
///
/// cost: O(n^2) time, memory and calls of `cost`
/// empty: `n == 0` gives `[[0]]`
/// ties: only the value is kept; which split attains it does not matter
/// order: `dp[i][j]` is row `i`, column `j`
/// traps: when `n < 0`
/// example: `knuth(2, (i, j) => j - i)[0][2] == 2`
/// example: `knuth(3, (i, j) => j - i)[0][3] == 5`
/// example: `knuth(1, (i, j) => j - i) == [[0, 0], [0, 0]]`
pub fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>> = todo

/// `f(k)` for a convex `f` known only through its penalized form: `solve(λ)` returns the least
/// `f(c) + λ * c` over every count `c`, and the least `c` attaining it.
///
/// The search finds the least `λ` whose count is at most `k` and returns `val - λ * k`, which is
/// exact because `f` is convex. Law: `f` is convex (`std.spec.convexity_break`), the counts fall as
/// `λ` grows, and `solve` reports the least optimal count. `L` is the value type (`Int`, `I128`).
/// Without `lambdas` the range is found by doubling out from `-1..=1`; pass it when `λ * k` could
/// overflow `L`.
///
/// cost: O(log lambdas.len) calls of `solve` (fewer once a count is exactly `k`); O(log |λ*|) more without `lambdas`
/// empty: an empty `lambdas` traps
/// order: `lambdas` is searched as an increasing range; one value is returned
/// ties: among counts with equal penalized value, `solve` must report the least; `aliens` relies on it
/// traps: when `k < 0`, when `lambdas` is empty, or when no `λ` in `lambdas` gives a count of at most `k`
/// example: `aliens(5, lam => (0..=10).map(c => (val: (c - 3) * (c - 3) + lam * c, count: c)).min_by(p => (p.val, p.count))!) == 4`
pub fn aliens<L>(k: Int, solve: L -> (val: L, count: Int), lambdas: ?Range<L> = None) -> L = todo

// The penalties around the answer: the counts fall as the penalty grows, so
// double until the count is at most `k` above and at least `k` below.
fn penalties<L>(k: Int, solve: L -> (val: L, count: Int)) -> Range<L> = todo

/// Every subset sum of `xs`, one per subset (repeats kept), sorted ascending: the half of a
/// meet-in-the-middle. Built by merging, not by sorting.
///
/// cost: O(2^n) time and memory for n = `xs.len`
/// empty: `[]` gives `[0]`
/// order: ascending; equal sums are indistinguishable
/// traps: when a sum overflows `Int`
/// example: `subset_sums([1, 2]) == [0, 1, 2, 3]`
/// example: `subset_sums([1, 1]) == [0, 1, 1, 2]`
pub fn subset_sums(xs: List<Int>) -> List<Int> = todo

struct Counted(k: Int, m: Int, used: Int):
    fn at(self, x: Int) -> (cost: Int, used: Int) = todo

// Online LARSCH over nodes 0..n; the engine of `monge_path` and `monge_paths`.
fn larsch<K>(n: Int, w: (Int, Int) -> K) -> (val: List<K>, prev: List<Int>) = todo

/// The shortest path from `nodes.first` to `nodes.last` along increasing edges
/// `nodes[j] -> nodes[i]`, `j < i`, of weight `cost(nodes[j], nodes[i])`: its total and its number
/// of edges, the shape `aliens` asks `solve` for
/// (`aliens(k, solve: pen => monge_path(cuts, cost: (j, i) => w(j, i) + pen))`).
///
/// Law: `cost` is Monge along `nodes`, `cost(a, c) + cost(b, d) <= cost(a, d) + cost(b, c)` for
/// `a <= b <= c <= d` (`std.spec.monge_break`). A single node is a path of no edges, total `0`.
///
/// cost: O(n) calls of `cost` (online LARSCH), O(n) memory, for n = `nodes.len`
/// empty: an empty `nodes` traps; one node gives `(val: 0, count: 0)`
/// ties: among paths of equal total the one found by LARSCH, not necessarily the fewest edges
/// traps: when `nodes` is empty
/// example: `monge_path([0, 1, 2], (a, b) => (b - a) * (b - a)) == (val: 2, count: 2)`
pub fn monge_path<N, K>(nodes: List<N>, cost: (N, N) -> K) -> (val: K, count: Int) = todo

/// Every shortest path from `nodes.first`: `val[i]` is the least total weight to `nodes[i]` and
/// `prev[i]` the index before it on that path (`val[0] == 0`, `prev[0] == 0`). Same law as
/// `monge_path`; follow `prev` back from `i` to recover a path.
///
/// cost: O(n) calls of `cost` (online LARSCH), O(n) memory, for n = `nodes.len`
/// empty: an empty `nodes` traps
/// ties: among equal totals `prev[i]` is the predecessor found by LARSCH
/// order: both lists are indexed like `nodes`
/// traps: when `nodes` is empty
/// example: `monge_paths([0, 1, 2], (a, b) => (b - a) * (b - a)) == (val: [0, 1, 2], prev: [0, 0, 1])`
pub fn monge_paths<N, K>(nodes: List<N>, cost: (N, N) -> K) -> (val: List<K>, prev: List<Int>) = todo

/// O(1) per step; the next cut's optimum after O(1) amortized calls of `cost`.
/// `monge_path` one node at a time, for a DP whose next weight depends on the
/// optimum just found. `next()` returns the optimum at the next node and the
/// node before it; `val(i)` reads any node already reached.
pub struct MongePath<N, K>(nodes: List<N>, cost: (N, N) -> K):
    /// Reach the next node: return the optimum at node `len` and the index of the node before it, then
    /// grow `len` by one. `cost` may read values already reached through `val`, so the weights can
    /// depend on earlier optima.
    ///
    /// cost: O(1) amortized calls of `cost`
    /// ties: among equal totals the predecessor found by LARSCH
    /// traps: when every node has been reached (`len == nodes.len`)
    /// example: `var p = MongePath([0, 1, 2], (a, b) => (b - a) * (b - a)); p.next(); p.next() == (val: 2, prev: 1)`
    pub fn next(mut self) -> (val: K, prev: Int) = todo
    /// The optimum at node `i`, which must already be reached; `val(0)` is 0.
    ///
    /// cost: O(1)
    /// traps: when `i < 0` or `i >= len`
    /// example: `var p = MongePath([0, 1, 2], (a, b) => (b - a) * (b - a)); p.next(); p.val(1) == 1`
    pub fn val(self, i: Int) -> K = todo
    /// The number of nodes reached so far, `nodes.first` included, so a fresh `MongePath` has `len == 1`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MongePath([0, 1, 2], (a, b) => (b - a) * (b - a)).len == 1`
    @get pub fn len(self) -> Int = todo

/// For each row of the implicit `rows` x `cols` matrix `f`, the least `f(row, col)` and its
/// leftmost `col`.
///
/// Law: `f` is totally monotone (true when Monge; `std.spec.monge_break` checks that). Prefer
/// `monotone_minima` when `f` may be `None`: it needs only monotone argmins.
///
/// cost: O(rows + cols) calls of `f`
/// empty: `rows == 0` gives `[]`; `cols == 0` traps (a row has no minimum)
/// ties: the leftmost `col` among equal minima
/// order: one entry per row, row order
/// traps: when `rows < 0`, or `cols <= 0` with `rows > 0`
/// example: `smawk(2, 3, (i, j) => (j - i) * (j - i)) == [(val: 0, at: 0), (val: 0, at: 1)]`
pub fn smawk(rows: Int, cols: Int, f: (Int, Int) -> Int) -> List<(val: Int, at: Int)> = todo

/// O(1) amortized per `add` and per `at_next`, O(log n) per `at`.
/// The convex hull trick for lines added in slope order: falling slopes for
/// `pick: min`, rising for `max` (an out-of-order slope traps). The online
/// form of `LiChao` when slopes arrive sorted and no x-range is known.
/// Each line may carry a `tag` (a photo count, a parent index); `best_next` returns the winning
/// line's tag, and equal values go to the smaller tag, which is the tie rule `aliens` needs.
pub struct MonotoneHull<P>(pick: (Int, Int) -> Int = min):
    /// Add `line`, with `tag`. Slopes must arrive in order: non-increasing for `pick: min`,
    /// non-decreasing for `max`. A line of equal slope keeps only the better intercept, and lines that
    /// are never best are dropped.
    ///
    /// cost: O(1) amortized
    /// traps: when the slope goes back
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.add(Line(-1, 3)); h.at(0) == Some(0) and h.at(2) == Some(1)`
    pub fn add(mut self, line: Line, tag: P = ()) = todo
    /// The best value at `x` with the tag of the line attaining it, for an `x` that never decreases
    /// between calls (a pointer walks the hull). `None` when empty.
    ///
    /// cost: O(1) amortized
    /// ties: equal values go to the smaller tag (the rule `aliens` needs)
    /// traps: when `x` is less than the previous `best_next` or `at_next` argument
    /// example: `var h = MonotoneHull<Int>(min); h.add(Line(1, 0), tag: 7); h.add(Line(-1, 2), tag: 3); h.best_next(1) == Some((val: 1, tag: 3))`
    pub fn best_next(mut self, x: Int) -> ?(val: Int, tag: P) = todo
    /// The best value at any `x`, by binary search on the hull; `None` when empty.
    ///
    /// cost: O(log n)
    /// ties: only the value is kept
    /// traps: never
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.add(Line(-1, 3)); h.at(5) == Some(-2)`
    pub fn at(self, x: Int) -> ?Int = todo
    /// `at(x)` for an `x` that never decreases between calls; `None` when empty.
    ///
    /// cost: O(1) amortized
    /// ties: only the value is kept
    /// traps: when `x` is less than the previous `best_next` or `at_next` argument
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.add(Line(-1, 3)); h.at_next(0) == Some(0) and h.at_next(2) == Some(1)`
    pub fn at_next(mut self, x: Int) -> ?Int = todo
    /// The number of lines kept on the hull (dominated lines are not counted).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.add(Line(-1, 3)); h.len == 2`
    @get pub fn len(self) -> Int = todo
    /// Remove every line and reset the `at_next` pointer, keeping `pick`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MonotoneHull<()>(min); h.add(Line(2, 0)); h.clear(); h.at(0) == None`
    pub fn clear(mut self) = todo

/// O(log n) amortized per `add`, O(log n) per `at`.
/// The convex hull trick with lines in any order and queries at any `x`,
/// no coordinate range needed (KACTL's line container). Use `LiChao` for
/// segment adds or non-line functions.
pub struct LineContainer(pick: (Int, Int) -> Int = min):
    /// Add `line`; slopes may arrive in any order. Lines that are never best are dropped.
    ///
    /// cost: O(log n) amortized
    /// traps: never
    /// example: `var c = LineContainer(max); c.add(Line(1, 0)); c.add(Line(-1, 4)); c.at(0) == Some(4) and c.at(3) == Some(3)`
    pub fn add(mut self, line: Line) = todo
    /// The best value at `x`, or `None` when empty.
    ///
    /// cost: O(log n)
    /// ties: only the value is kept
    /// traps: never
    /// example: `var c = LineContainer(min); c.add(Line(1, 0)); c.add(Line(-1, 4)); c.at(0) == Some(0) and c.at(3) == Some(1)`
    pub fn at(self, x: Int) -> ?Int = todo
    /// The line attaining `at(x)`, or `None` when empty.
    ///
    /// cost: O(log n)
    /// ties: unspecified among lines with equal values at `x`
    /// traps: never
    /// example: `var c = LineContainer(max); c.add(Line(1, 0)); c.add(Line(-1, 4)); c.line_at(3) == Some(Line(1, 0))`
    pub fn line_at(self, x: Int) -> ?Line = todo
    /// The number of lines kept (dominated lines are not counted).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var c = LineContainer(max); c.add(Line(1, 0)); c.add(Line(-1, 4)); c.len == 2`
    @get pub fn len(self) -> Int = todo

/// The length of the longest increasing subsequence ending at each position, compared by `by`;
/// `strict: false` allows equal neighbours (non-decreasing). `xs.lis_ends().max()` is the LIS
/// length; `xs.lis()` in `std.seq` gives one.
///
/// cost: O(n log n) time, O(n) memory, one `by` call per element
/// empty: an empty array gives an empty array
/// order: same domain and positions as `self`
/// traps: never
/// example: `[3, 1, 4, 1, 5].lis_ends() == [1, 1, 2, 1, 3]`
/// example: `[1, 1, 2].lis_ends(strict: false) == [1, 2, 3]`
/// example: `[3, 1, 2].lis_ends(by: x => -x) == [1, 2, 2]`
pub fn lis_ends<D, T, K>(self: [D]T, strict: Bool = true, by: T -> K = x => x) -> [D]Int = todo


/// Every mask of `bits` bits containing `m`, increasing from `m` to `2^bits - 1`.
///
/// cost: O(2^(bits - popcount(m)))
/// empty: never empty; `m` itself comes first
/// order: increasing
/// traps: when `bits < 0` or `m` is not in `0..2^bits`
/// example: `supermasks(0b01, 2).collect() == [1, 3]`
pub fn supermasks(m: Int, bits: Int) -> yields Int = todo

/// Every mask of `bits` bits with exactly `k` bits set, increasing (Gosper's hack).
///
/// cost: O(C(bits, k))
/// empty: nothing when `k > bits`; `k == 0` yields only `0`
/// order: increasing
/// traps: when `bits < 0` or `k < 0`
/// example: `masks_of_size(3, 2).collect() == [3, 5, 6]`
pub fn masks_of_size(bits: Int, k: Int) -> yields Int = todo

/// `out[mask]` is the fold under `op` of `xs[i]` for `i` in `mask`, in increasing `i`;
/// `out[0]` is `op`'s identity (W10).
///
/// cost: O(2^n) time and memory, one `op` call per non-empty mask, for n = `xs.len`
/// empty: `[]` gives `[identity]`
/// order: indexed by mask; within a mask, elements fold in increasing index
/// traps: never
/// example: `subset_folds([1, 2, 4], +) == [0, 1, 2, 3, 4, 5, 6, 7]`
pub fn subset_folds<T, M>(xs: List<T>, op: M) -> List<T> = todo

/// Depth-first backtracking from `start`, where `next(s)` yields the states one step on. Every
/// state reached is yielded as a `Step` with `.state` and `.depth` (`start` at depth 0), in
/// preorder: a state before its children, children in the order `next` yields them.
///
/// Calling `.prune()` on the current step skips every state below it; stop the whole search with
/// `break` or `return` in the `for` body (decision 126). States are not deduplicated.
///
/// cost: O(visited) steps plus the cost of `next`
/// order: preorder; children in `next` order
/// traps: never
/// example: `backtrack(1, x => if x < 4: [2 * x, 2 * x + 1] else: []).map(s => s.state).collect() == [1, 2, 4, 5, 3, 6, 7]`
/// example: `backtrack(1, x => if x < 4: [2 * x, 2 * x + 1] else: []).map(s => s.depth).collect() == [0, 1, 2, 2, 1, 2, 2]`
pub fn backtrack<S>(start: S, next: S -> yields S) -> yields Step<S> = todo

/// One state reached by `backtrack`.
pub struct Step<S>(state: S, depth: Int):
    /// Skip every state below this one: `backtrack` does not call `next` on it. Has effect only on
    /// the step just yielded, before the `for` body continues.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: with `next` as `x => if x < 4: [2 * x, 2 * x + 1] else: []`, a `for s in backtrack(1, next)` body that calls `s.prune()` when `s.state == 2` sees the states `[1, 2, 3, 6, 7]`
    pub fn prune(self) = todo

@test
fn li_chao_matches_a_scan_over_lines_and_segments() = todo

@test
fn dnc_layers_match_the_quadratic_dp() = todo

@test
fn knuth_matches_the_cubic_interval_dp() = todo

@test
fn aliens_finds_the_exact_count_optimum() = todo

@test
fn aliens_searches_a_wide_i128_slope_range() = todo

@test
fn subset_sums_are_every_sum_sorted() = todo

@test
fn monge_paths_match_the_quadratic_dp() = todo
