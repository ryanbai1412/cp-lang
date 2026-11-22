//! Sparse table: O(1) range folds of a fixed array.
use std.math.{gcd}
use std.seq.{ends}

/// `SparseTable<V, Int>(min, a)`: built in O(n log n) time and memory, `fold` is O(1).
/// The operation must be associative and idempotent (`op(a, a) == a`): `min`, `max`, `gcd`,
/// `&`, `|`, or any custom `(T, T) -> T` — no identity is required because `fold` asserts
/// the range is non-empty and returns `T` directly. Sums are not idempotent; use `Fenwick` or `prefix` for those.
pub struct SparseTable<D, T>(op: (T, T) -> T):
    n: Int
    table: List<T>

    /// Build a sparse table for the idempotent, associative `op` over the values of `a`. Level `k`
    /// holds the fold of every window of length `2^k`, so any non-empty range is the fold of two
    /// overlapping windows. The table copies `a` and keeps its domain `D`.
    ///
    /// cost: O(|D| log |D|) time and memory, `op` called O(|D| log |D|) times
    /// empty: an empty `a` builds an empty table; every `fold` on it traps, `try_fold` is `None`
    /// order: the table keeps the domain `D` of `a`
    /// traps: never (a non-idempotent or non-associative `op` gives unspecified folds; check with `idempotent`)
    /// example: `SparseTable(min, [3, 1, 4]).fold(0..3) == 1`
    pub fn new(op: (T, T) -> T, a: [D]T) -> SparseTable<D, T> = todo

    /// The fold of `op` over the positions of the non-empty range `r`, as two overlapping
    /// power-of-two windows combined with one `op` call.
    ///
    /// cost: O(1)
    /// empty: traps; use `try_fold` when `r` may be empty
    /// traps: when `r` is empty or reaches outside `D`
    /// example: `SparseTable(max, [3, 1, 4]).fold(0..2) == 3`
    /// example: `SparseTable(min, [3, 1, 4]).fold(1..2) == 1`
    pub fn fold(self, r: Range<D>) -> T = todo

    /// The fold of `r` like `fold`, or `None` when `r` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `SparseTable(min, [3, 1, 4]).try_fold(1..1) == None`
    /// example: `SparseTable(min, [3, 1, 4]).try_fold(1..3) == Some(1)`
    pub fn try_fold(self, r: Range<D>) -> ?T = todo

    /// The original value at position `i` (level 0 of the table).
    ///
    /// cost: O(1)
    /// traps: when `i` is outside `D`
    /// example: `SparseTable(min, [3, 1, 4]).get(2) == 4`
    pub fn get(self, i: D) -> T = todo

    /// The fold of every position of `D`; `fold(D.all)`.
    ///
    /// cost: O(1)
    /// empty: traps on an empty domain
    /// traps: when `D` is empty
    /// example: `SparseTable(gcd, [4, 6, 10]).all == 2`
    @get pub fn all(self) -> T = todo

/// Whether `op(x, x) == x` for every sample `x` in `xs`: the law a sparse table needs on top of
/// associativity. A sample check, not a proof; associativity is not tested.
///
/// cost: O(n) calls of `op`
/// empty: `true`
/// order: samples are checked in list order; the first failure decides
/// traps: never (`op` may trap)
/// example: `idempotent(max, [1, 5]) == true`
/// example: `idempotent((a, b) => a + b, [0, 1]) == false`
pub fn idempotent<T>(op: (T, T) -> T, xs: List<T>) -> Bool = todo

/// Disjoint sparse table: O(n log n) build and memory, O(1) range fold for ANY associative
/// operation — no idempotence and no inverse required (matrix products, `Mod<P>` products,
/// `MaxSub`, string concatenation). Asserts `not r.is_empty`.
pub struct DisjointSparseTable<D, T>(op: (T, T) -> T):
    n: Int
    table: List<T>

    /// Build a disjoint sparse table for the associative `op` over `a`. Each level splits the domain
    /// into blocks and stores folds outward from each block's middle, so a range is answered by
    /// combining one suffix fold and one prefix fold; no idempotence, identity or inverse is needed.
    ///
    /// cost: O(|D| log |D|) time and memory, `op` called O(|D| log |D|) times
    /// empty: an empty `a` builds an empty table; every `fold` on it traps, `try_fold` is `None`
    /// order: the table keeps the domain `D` of `a`; folds run in domain order
    /// traps: never (a non-associative `op` gives unspecified folds)
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).fold(0..3) == 8`
    pub fn new(op: (T, T) -> T, a: [D]T) -> DisjointSparseTable<D, T> = todo

    /// The fold of `op` over the non-empty range `r`, left to right in domain order, so a
    /// non-commutative `op` sees its operands in position order.
    ///
    /// cost: O(1), at most one `op` call
    /// empty: traps; use `try_fold` when `r` may be empty
    /// traps: when `r` is empty or reaches outside `D`
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).fold(1..3) == 5`
    /// example: `DisjointSparseTable((a, b) => a, [7, 8, 9]).fold(1..3) == 8`
    pub fn fold(self, r: Range<D>) -> T = todo

    /// The fold of `r` like `fold`, or `None` when `r` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).try_fold(2..2) == None`
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).try_fold(0..2) == Some(4)`
    pub fn try_fold(self, r: Range<D>) -> ?T = todo

    /// The original value at position `i`.
    ///
    /// cost: O(1)
    /// traps: when `i` is outside `D`
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).get(0) == 3`
    pub fn get(self, i: D) -> T = todo

    /// The fold of every position of `D`, left to right; `fold(D.all)`.
    ///
    /// cost: O(1)
    /// empty: traps on an empty domain
    /// traps: when `D` is empty
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).all == 8`
    @get pub fn all(self) -> T = todo

/// O(|R| * |C| * log |R| * log |C|) build and memory; O(1) rectangle fold for any associative,
/// idempotent operation (`min`, `max`, `gcd`, `&`, `|`).
pub struct SparseTable2D<R, C, T>(op: (T, T) -> T):
    table: List<T>

    /// Build a 2-D sparse table for the idempotent, associative `op` over the grid `a`: one
    /// `SparseTable` level per power-of-two row height and column width. Keeps the domains `R`, `C`.
    ///
    /// cost: O(|R| |C| log |R| log |C|) time and memory
    /// empty: an empty grid builds an empty table; every `fold` on it traps
    /// order: the table keeps the domains `R` and `C` of `a`
    /// traps: never (a non-idempotent `op` gives unspecified folds)
    /// example: `SparseTable2D(max, [2, 2] (r, c) => r * 2 + c).fold(0..2, 0..2) == 3`
    pub fn new(op: (T, T) -> T, a: [R, C]T) -> SparseTable2D<R, C, T> = todo

    /// The fold of `op` over the rectangle `rows` by `cols`, both non-empty: four overlapping
    /// power-of-two blocks combined with `op`.
    ///
    /// cost: O(1)
    /// empty: traps when either range is empty; use `try_fold`
    /// traps: when either range is empty or reaches outside its domain
    /// example: `SparseTable2D(min, [2, 2] (r, c) => r * 2 + c).fold(1..2, 0..2) == 2`
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T = todo

    /// The rectangle fold like `fold`, or `None` when `rows` or `cols` is empty.
    ///
    /// cost: O(1)
    /// empty: `None` when either range is empty
    /// traps: when a range reaches outside its domain on a non-empty rectangle
    /// example: `SparseTable2D(min, [2, 2] (r, c) => r * 2 + c).try_fold(0..2, 1..1) == None`
    /// example: `SparseTable2D(min, [2, 2] (r, c) => r * 2 + c).try_fold(0..2, 1..2) == Some(1)`
    pub fn try_fold(self, rows: Range<R>, cols: Range<C>) -> ?T = todo

    /// The original cell value `a[r, c]`.
    ///
    /// cost: O(1)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `SparseTable2D(min, [2, 2] (r, c) => r * 2 + c).get(1, 0) == 2`
    pub fn get(self, r: R, c: C) -> T = todo

/// O(|D|) build and memory; O(1) range min/max query via block-of-64 bitmask RMQ (Four Russians).
/// Useful for O(1) LCA and memory-tight static RMQ where O(n log n) space is too large.
pub struct LinearRmq<D, T>(before: (T, T) -> Bool):
    n: Int

    /// Build a linear-space range-best structure over `a`, where `before(x, y)` means `x` is better
    /// than `y` (default `<`, so the best is the minimum). Blocks of 64 positions carry a sparse
    /// table over block bests and an in-block bitmask stack, so queries never scan.
    ///
    /// cost: O(|D|) time and memory
    /// empty: an empty `a` builds an empty structure; `argbest` and `best` on it trap
    /// order: the structure keeps the domain `D` of `a`
    /// traps: never (`before` must be a strict weak order, or results are unspecified)
    /// example: `LinearRmq([3, 1, 4]).best(0..3) == 1`
    pub fn new(a: [D]T, before: (T, T) -> Bool = (x, y) => x < y) -> LinearRmq<D, T> = todo

    /// The position of the best element of the non-empty range `r` under `before`; the leftmost
    /// when several are equally best (neither is `before` the other).
    ///
    /// cost: O(1)
    /// empty: traps
    /// ties: the first in key order
    /// traps: when `r` is empty or reaches outside `D`
    /// example: `LinearRmq([3, 1, 1, 4]).argbest(0..4) == 1`
    /// example: `LinearRmq([3, 1, 4], before: (x, y) => x > y).argbest(0..3) == 2`
    pub fn argbest(self, r: Range<D>) -> D = todo

    /// The value of the best element of the non-empty range `r` under `before`; `a[argbest(r)]`.
    ///
    /// cost: O(1)
    /// empty: traps; use `try_best` when `r` may be empty
    /// ties: equal-best values are interchangeable; the value at the leftmost is returned
    /// traps: when `r` is empty or reaches outside `D`
    /// example: `LinearRmq([3, 1, 4], before: (x, y) => x > y).best(0..3) == 4`
    pub fn best(self, r: Range<D>) -> T = todo

    /// The value of the best element of `r` like `best`, or `None` when `r` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: the value at the leftmost equal-best position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `LinearRmq([3, 1]).try_best(1..1) == None`
    /// example: `LinearRmq([3, 1]).try_best(0..2) == Some(1)`
    pub fn try_best(self, r: Range<D>) -> ?T = todo

@test
fn range_min_matches_brute_force() = todo

@test
fn gcd_max_bitwise_and_custom_semigroup_tables() = todo

@test
fn idempotence_separates_min_from_sum() = todo

@test
fn disjoint_sparse_table_handles_non_idempotent_non_commutative_folds() = todo
