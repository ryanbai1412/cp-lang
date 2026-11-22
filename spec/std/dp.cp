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
/// O(1).
    /// Evaluate the line `k * x + m` at `x`. O(1).
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
    // a cell's children; 0 is none, since the root is no cell's child
    left: List<Int>
    right: List<Int>

/// O(1).
    /// Create a Li Chao tree over integer x-domain `over` with comparator `pick`. O(1).
    pub fn new(pick: (K, K) -> K, over: Range<Int>) -> LiChao<L, K> = todo

/// O(1) amortized.
    /// Forget every line. O(1) amortized.
    pub fn clear(mut self) = todo

/// O(log n).
    /// The best value of a line at `x`, or `none` when no line covers `x`.
    pub fn at(self, x: Int) -> ?K = todo

/// O(log n).
    /// The line that is best at `x`, or `none` when no line covers `x`.
    pub fn line_at(self, x: Int) -> ?L = todo

/// O(log n), O(log^2 n) on a segment.
    /// Add `line` on all of `xs`, or only on the coordinates `on`.
    pub fn add(mut self, line: L, on: ?Range<Int> = none) = todo

    fn wins(self, a: K, b: K) -> Bool = todo

    fn kid(mut self, i: Int, right_side: Bool) -> Int = todo

    // Keep the line better at the middle of `l..=r` here; the other one is
    // better on at most one half, and goes down into it.
    fn settle(mut self, at: Int, lo: Int, hi: Int, line: L) = todo

/// O((rows + cols) log rows) calls of `f`.
/// For each row, the least `f(row, col)` and its leftmost `col`, or `none`
/// where `f` is `none` on the whole row. Law: the leftmost argmin does not
/// decrease from row to row (true when `f` is Monge; `std.spec.monge_break`
/// checks it). O((rows + cols) log rows) calls of `f`.
pub fn monotone_minima(rows: Int, cols: Int, f: (Int, Int) -> ?Int) -> List<?(val: Int, at: Int)> = todo

/// O(n log n) calls of `cost`.
/// One layer of a partition DP: `next[i]` is the least `prev[j] + cost(j, i)`
/// over `j < i` with `prev[j]` present, so `next[0]` is `none`. Indices are
/// cuts `0..prev.len`. Law: `cost` satisfies the quadrangle inequality
/// (`std.spec.dnc_break` checks it), so the best `j` moves right with `i`.
/// O(n log n) calls of `cost` for n = `prev.len`.
pub fn dnc_layer(prev: List<?Int>, cost: (Int, Int) -> Int) -> List<?Int> = todo

/// O(n^2) time and memory.
/// The interval DP over cuts `0..=n`: `dp[i][i + 1] = 0` and `dp[i][j] =
/// cost(i, j) + min over i < k < j of dp[i][k] + dp[k][j]`. Law: `cost` is
/// monotone under inclusion and satisfies the quadrangle inequality
/// (`std.spec.knuth_break` checks the split monotonicity it implies), so the
/// best split of `[i, j)` lies between those of `[i, j - 1)` and `[i + 1, j)`.
/// O(n^2) time and memory.
pub fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>> = todo

/// O(log lambdas.len) calls of `solve`, fewer once a count is exactly `k`.
/// `f(k)` for a convex `f` known only through its penalized form:
/// `solve(λ)` is the least `f(c) + λ * c` over every count `c`, and the least
/// `c` attaining it. Law: `f` is convex (`std.spec.convexity_break`), and the
/// first `λ` of `lambdas` gives a count of at most `k` somewhere in the range
/// (the counts fall as `λ` grows). `L` is the value type (`Int`, `I128`).
/// Without `lambdas` the range is found by doubling out from `-1..=1`
/// (O(log |λ*|) more calls); pass it when `λ * k` could overflow `L`.
pub fn aliens<L>(k: Int, solve: L -> (val: L, count: Int), lambdas: ?Range<L> = none) -> L = todo

// The penalties around the answer: the counts fall as the penalty grows, so
// double until the count is at most `k` above and at least `k` below.
fn penalties<L>(k: Int, solve: L -> (val: L, count: Int)) -> Range<L> = todo

/// O(2^n) time and memory.
/// Every subset sum of `xs`, sorted: the half of a meet-in-the-middle.
/// O(2^n) time and memory, by merging.
pub fn subset_sums(xs: List<Int>) -> List<Int> = todo

struct Counted(k: Int, m: Int, used: Int):
    fn at(self, x: Int) -> (cost: Int, used: Int) = todo

// Online LARSCH over nodes 0..n; the engine of `monge_path` and `monge_paths`.
fn larsch<K>(n: Int, w: (Int, Int) -> K) -> (val: List<K>, prev: List<Int>) = todo

/// O(n) calls of `cost` (online LARSCH), O(n) memory, for n = `nodes.len`.
/// The shortest path from `nodes.first` to `nodes.last` along increasing
/// edges `nodes[j] -> nodes[i]`, `j < i`, of weight `cost(nodes[j], nodes[i])`:
/// its total and its number of edges, the shape `aliens` asks `solve` for
/// (`aliens(k, solve: pen => monge_path(cuts, cost: (j, i) => w(j, i) + pen))`).
/// Law: `cost` is Monge along `nodes`, `cost(a, c) + cost(b, d) <= cost(a, d) +
/// cost(b, c)` for `a <= b <= c <= d` (`std.spec.monge_break`); on a tie the
/// fewest edges is not guaranteed.
pub fn monge_path<N, K>(nodes: List<N>, cost: (N, N) -> K) -> (val: K, count: Int) = todo

/// O(n) calls of `cost` (online LARSCH), O(n) memory, for n = `nodes.len`.
/// Every shortest path from `nodes.first`: `val[i]` is the least total weight
/// to `nodes[i]` and `prev[i]` the index before it on that path (`prev[0] ==
/// 0`). Same law as `monge_path`.
pub fn monge_paths<N, K>(nodes: List<N>, cost: (N, N) -> K) -> (val: List<K>, prev: List<Int>) = todo

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
/// Each line may carry a `tag` (a photo count, a parent index); `best_next` returns the winning
/// line's tag, and equal values go to the smaller tag, which is the tie rule `aliens` needs.
pub struct MonotoneHull<P>(pick: (Int, Int) -> Int = min):
    /// O(1) amortized; the slope must not go back.
    pub fn add(mut self, line: Line, tag: P = ()) = todo
    /// O(1) amortized; the best value at an `x` that never decreases between calls, with the
    /// tag of the line attaining it. `none` when empty.
    pub fn best_next(mut self, x: Int) -> ?(val: Int, tag: P) = todo
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
