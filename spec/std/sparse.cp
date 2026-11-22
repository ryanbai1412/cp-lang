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

    /// Build an idempotent range-query sparse table for `op` over `a`. O(|D| log |D|). Law: `op(x, x) == x`.
    pub fn new(op: (T, T) -> T, a: [D]T) -> SparseTable<D, T> = todo

    /// The fold of a non-empty range. O(1). Asserts `not r.is_empty`.
    pub fn fold(self, r: Range<D>) -> T = todo

    /// O(1). Fold of `r`, or `None` when `r` is empty.
    pub fn try_fold(self, r: Range<D>) -> ?T = todo

    /// O(1). Original value at position `i`.
    pub fn get(self, i: D) -> T = todo

    /// O(1). Fold of the entire non-empty domain.
    pub @get fn all(self) -> T = todo

/// Is `op` idempotent on these samples? The law a sparse table needs alongside associativity.
pub fn idempotent<T>(op: (T, T) -> T, xs: List<T>) -> Bool = todo

/// Disjoint sparse table: O(n log n) build and memory, O(1) range fold for ANY associative
/// operation — no idempotence and no inverse required (matrix products, `Mod<P>` products,
/// `MaxSub`, string concatenation). Asserts `not r.is_empty`.
pub struct DisjointSparseTable<D, T>(op: (T, T) -> T):
    n: Int
    table: List<T>

    /// Build an associative O(1)-query disjoint sparse table for `op` over `a`. O(|D| log |D|).
    pub fn new(op: (T, T) -> T, a: [D]T) -> DisjointSparseTable<D, T> = todo

    /// Fold of a non-empty range `r`, left to right. O(1). Asserts `not r.is_empty`.
    pub fn fold(self, r: Range<D>) -> T = todo

    /// O(1). Fold of `r`, or `None` when `r` is empty.
    pub fn try_fold(self, r: Range<D>) -> ?T = todo

    /// O(1). Original value at position `i`.
    pub fn get(self, i: D) -> T = todo

    /// O(1). Fold of the entire non-empty domain.
    pub @get fn all(self) -> T = todo

/// O(|R| * |C| * log |R| * log |C|) build and memory; O(1) rectangle fold for any associative,
/// idempotent operation (`min`, `max`, `gcd`, `&`, `|`).
pub struct SparseTable2D<R, C, T>(op: (T, T) -> T):
    table: List<T>

    /// O(|R| * |C| * log |R| * log |C|). Build a 2D idempotent sparse table over `a`.
    pub fn new(op: (T, T) -> T, a: [R, C]T) -> SparseTable2D<R, C, T> = todo

    /// O(1). Fold of a non-empty rectangle `rows` by `cols`. Asserts both ranges are non-empty.
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T = todo

    /// O(1). Fold of `rows` by `cols`, or `None` when either range is empty.
    pub fn try_fold(self, rows: Range<R>, cols: Range<C>) -> ?T = todo

    /// O(1). Original cell value `a[r, c]`.
    pub fn get(self, r: R, c: C) -> T = todo

/// O(|D|) build and memory; O(1) range min/max query via block-of-64 bitmask RMQ (Four Russians).
/// Useful for O(1) LCA and memory-tight static RMQ where O(n log n) space is too large.
pub struct LinearRmq<D, T>(before: (T, T) -> Bool):
    n: Int

    /// O(|D|). Build a linear-space O(1)-query RMQ over `a` ordered by `before` (default `<` for min).
    pub fn new(a: [D]T, before: (T, T) -> Bool = (x, y) => x < y) -> LinearRmq<D, T> = todo

    /// O(1). Position of the best element in non-empty range `r` (leftmost on tie).
    pub fn argbest(self, r: Range<D>) -> D = todo

    /// O(1). Value of the best element in non-empty range `r`.
    pub fn best(self, r: Range<D>) -> T = todo

    /// O(1). Value of the best element in `r`, or `None` when `r` is empty.
    pub fn try_best(self, r: Range<D>) -> ?T = todo

@test
fn range_min_matches_brute_force() = todo

@test
fn gcd_max_bitwise_and_custom_semigroup_tables() = todo

@test
fn idempotence_separates_min_from_sum() = todo

@test
fn disjoint_sparse_table_handles_non_idempotent_non_commutative_folds() = todo
