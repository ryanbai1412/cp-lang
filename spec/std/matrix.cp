//! Dense matrices, exact and floating-point Gaussian elimination, and GF(2) XOR bases.
//!
//! `Mat<T>` stores an `rows x cols` matrix in flat row-major order (`data[r * cols + c]`).
//! Exact elimination (`det`, `rank`, `solve`, `solve_space`, `inverse`) works over any field
//! type with `T.zero`, `T.one`, exact `==`, and `/` (such as `Mod<P>` for prime `P`).
//! Floating-point elimination uses partial pivoting with an explicit tolerance `eps` via
//! `det_real`, `rank_real`, `solve_real`, and `inverse_real`.
use std.algebra.{Monoid, Semiring}
use std.mod.{Mod}

/// A dense `rows x cols` matrix stored flat in row-major order.
pub struct Mat<T>:
    rows: Int
    cols: Int
    data: List<T>

    /// A `rows x cols` matrix filled with `fill`. O(rows * cols).
    pub fn new(rows: Int, cols: Int, fill: T) -> Mat<T> = todo

    /// A `rows x cols` zero matrix. O(rows * cols).
    pub fn zeros(rows: Int, cols: Int) -> Mat<T> = todo

    /// The `n x n` identity matrix. O(n^2).
    pub fn identity(n: Int) -> Mat<T> = todo

    /// Build a `rows x cols` matrix from a flat row-major list of length `rows * cols`. O(rows * cols).
    pub fn from_flat(rows: Int, cols: Int, data: List<T>) -> Mat<T> = todo

    /// Build a matrix from a list of equal-length rows. O(rows * cols).
    pub fn from_rows(grid: List<List<T>>) -> Mat<T> = todo

    /// Entry at `(r, c)`. O(1).
    pub fn get(self, r: Int, c: Int) -> T = todo

    /// Set entry at `(r, c)`. O(1).
    pub fn set(mut self, r: Int, c: Int, val: T) = todo

    /// Entry at `(r, c)`, lent in place: `m.at(i, j) += x`. O(1).
    pub fn at(self, r: Int, c: Int) lends mut T:
        lend mut data[r * cols + c]

    /// Transpose (`cols x rows`). O(rows * cols).
    pub fn transpose(self) -> Mat<T> = todo

    /// Matrix-vector product `self * v`. O(rows * cols).
    pub fn mul_vec(self, v: List<T>) -> List<T> = todo

    /// Matrix multiplication monoid for `n x n` matrices (usable in `SegTree`).
    pub fn product(n: Int) -> Monoid<Mat<T>> = todo

    /// Determinant over an exact field (such as `Mod<P>` for prime `P`). O(n^3).
    pub fn det(self) -> T = todo

    /// Rank over an exact field. O(rows * cols * min(rows, cols)).
    pub fn rank(self) -> Int = todo

    /// Solve `self * x = b` over an exact field. Returns `(particular, nullspace_basis)` or `none`.
    /// Every solution has the form `particular + sum(c_i * basis[i])`. O(rows * cols * min(rows, cols)).
    pub fn solve_space(self, b: List<T>) -> ?(List<T>, List<List<T>>) = todo

    /// Solve `self * x = b` over an exact field. Returns one solution (free variables 0) or `none`.
    pub fn solve(self, b: List<T>) -> ?List<T> = todo

    /// Multiplicative inverse of a square matrix over an exact field, or `none` if singular. O(n^3).
    pub fn inverse(self) -> ?Mat<T> = todo

    /// O(n * m * min(n, m)). Reduced row echelon form `(rref_mat, rank, pivot_cols)` over a field.
    pub fn rref(self) -> (mat: Mat<T>, rank: Int, pivots: List<Int>) = todo

    /// O(n^3 log m). Determinant modulo any (possibly composite) `m >= 1` via Euclidean elimination.
    pub fn det_mod(self, m: Int) -> Int = todo

    /// O(n^3). Characteristic polynomial `det(xI - self)` (lowest degree first) via Hessenberg reduction.
    pub fn char_poly(self) -> List<T> = todo

    /// O(n^3). Pfaffian of an even-order skew-symmetric matrix over a field.
    pub fn pfaffian(self) -> T = todo

    /// O(n^3). Adjugate matrix `adj(self)` (defined even when `rank == n - 1`).
    pub fn adjugate(self) -> Mat<T> = todo

    /// O(n * m * k). Matrix product over an arbitrary `Semiring<T>` (e.g. `(min, +)` tropical product).
    pub fn mul_semiring(self, o: Mat<T>, sr: Semiring<T>) -> Mat<T> = todo

impl Add for Mat<T>:
    fn add(a, b) = todo

impl Sub for Mat<T>:
    fn sub(a, b) = todo

impl Mul for Mat<T>:
    fn mul(a, b) = todo

impl Pow for Mat<T>:
    fn pow(a, e: Int) -> Mat<T> = todo

/// Determinant of a floating-point square matrix via partial pivoting with tolerance `eps`. O(n^3).
pub fn det_real(a: Mat<F64>, eps: F64 = 1e-9) -> F64 = todo

/// Rank of a floating-point matrix via partial pivoting with tolerance `eps`. O(r * c * min(r, c)).
pub fn rank_real(a: Mat<F64>, eps: F64 = 1e-9) -> Int = todo

/// Solve `a * x = b` over `F64` via partial pivoting with tolerance `eps`. Returns `none` if inconsistent.
pub fn solve_real(a: Mat<F64>, b: List<F64>, eps: F64 = 1e-9) -> ?List<F64> = todo

/// Inverse of a floating-point square matrix via partial pivoting with tolerance `eps`, or `none` if singular.
pub fn inverse_real(a: Mat<F64>, eps: F64 = 1e-9) -> ?Mat<F64> = todo

/// Linear basis over GF(2) for non-negative 63-bit integers (`0 <= x < 2^63`).
/// Every operation is O(W) where W <= 63 (`merge` is O(W * rank)).
pub struct XorBasis:
    basis: List<Int>
    size: Int

    /// Empty basis of rank 0. O(W).
    pub fn new() -> XorBasis = todo

    /// Number of linearly independent vectors in the basis. O(1).
    pub @get fn rank(self) -> Int = todo

    /// Insert `x >= 0` into the basis. Returns `true` iff the rank increased. O(W).
    pub fn add(mut self, x: Int) -> Bool = todo

    /// Reduce `x >= 0` against the basis (0 iff `x` is in the span). O(W).
    pub fn reduce(self, x: Int) -> Int = todo

    /// Whether `x >= 0` is a XOR-combination of basis vectors. O(W).
    pub fn contains(self, x: Int) -> Bool = todo

    /// Maximum value of `init ^ s` over all `s` in the span. O(W).
    pub fn max_xor(self, init: Int = 0) -> Int = todo

    /// Minimum value of `init ^ s` over all `s` in the span. O(W).
    pub fn min_xor(self, init: Int = 0) -> Int = todo

    /// The 0-indexed `k`-th smallest value in the span, or `none` if `k < 0` or `k >= 2^rank`. O(W^2).
    pub fn kth(self, k: Int) -> ?Int = todo

    /// Merge another basis into `self`. O(W * o.rank).
    pub fn merge(mut self, o: XorBasis) = todo

    /// Monoid under basis merge (usable in `SegTree`).
    pub fn monoid() -> Monoid<XorBasis> = todo

    /// O(62). Independent basis vectors currently stored.
    pub fn basis_vecs(self) -> List<Int> = todo

    /// O(62^2). Intersection of the two GF(2) vector subspaces.
    pub fn intersect(self, o: XorBasis) -> XorBasis = todo

fn merge_xor(a: XorBasis, b: XorBasis) -> XorBasis = todo

/// O(n^3 log max_exp) build, O(n^2 log e) query. Precomputed binary powers of a square matrix
/// for fast multi-query `A^e * v`.
pub struct MatPow<T>:
    n: Int

    /// O(n^3 log max_exp). Precompute `a^(2^k)` up to `max_exp`.
    pub fn new(a: Mat<T>, max_exp: Int) -> MatPow<T> = todo

    /// O(n^2 log e). Compute `a^e * v` for `0 <= e <= max_exp`.
    pub fn apply_pow(self, e: Int, v: List<T>) -> List<T> = todo

/// O(n * m / 64) storage. Bit-packed dense matrix over GF(2) with word-parallel elimination.
pub struct BitMat(rows: Int, cols: Int):
    /// O(rows * cols / 64). Zero GF(2) matrix.
    pub fn new(rows: Int, cols: Int) -> BitMat = todo

    /// O(n^2 / 64). `n x n` GF(2) identity matrix.
    pub fn identity(n: Int) -> BitMat = todo

    /// O(1). Read bit `(r, c)`.
    pub fn get(self, r: Int, c: Int) -> Bool = todo

    /// O(1). Write bit `(r, c)`.
    pub fn set(mut self, r: Int, c: Int, v: Bool) = todo

    /// O(n^3 / 64). Determinant over GF(2).
    pub fn det(self) -> Bool = todo

    /// O(n * m * min(n, m) / 64). Rank over GF(2).
    pub fn rank(self) -> Int = todo

    /// O(n * m * min(n, m) / 64). Any solution to `self * x == b` over GF(2), or `none`.
    pub fn solve(self, b: List<Bool>) -> ?List<Bool> = todo

    /// O(n * m * min(n, m) / 64). One solution `x0` and nullspace basis over GF(2), or `none`.
    pub fn solve_space(self, b: List<Bool>) -> ?(x0: List<Bool>, basis: List<List<Bool>>) = todo

    /// O(n^3 / 64). Inverse of a square GF(2) matrix, or `none`.
    pub fn inverse(self) -> ?BitMat = todo

    /// O(n * m * k / 64). Matrix product over GF(2).
    pub fn mul(self, o: BitMat) -> BitMat = todo

/// O(62) per push and query. Online prefix linear basis answering subarray XOR queries over any range `r`.
pub struct PrefixXorBasis:
    len: Int

    /// O(1). Empty prefix XOR basis.
    pub fn new() -> PrefixXorBasis = todo

    /// O(62). Append `x` at the next position, keeping rightmost-index basis vectors.
    pub fn push(mut self, x: Int) = todo

    /// O(62). Maximum `init ^ s` for `s` in the span of subarray `r`.
    pub fn max_xor(self, r: Range<Int>, init: Int = 0) -> Int = todo

    /// O(62). Whether `x` is in the span of subarray `r`.
    pub fn contains(self, r: Range<Int>, x: Int) -> Bool = todo

    /// O(62). Rank (dimension) of the span of subarray `r`.
    pub fn rank(self, r: Range<Int>) -> Int = todo

/// O(n * (entries.len + n)) expected. Determinant of a sparse `n x n` matrix via Wiedemann's algorithm.
pub fn sparse_det<P>(n: Int, entries: List<(Int, Int, Mod<P>)>) -> Mod<P> = todo

/// O(n * (entries.len + n)) expected. Solve `A x = b` for a sparse `n x n` matrix via Wiedemann's algorithm.
pub fn sparse_solve<P>(n: Int, entries: List<(Int, Int, Mod<P>)>, b: List<Mod<P>>) -> ?List<Mod<P>> = todo

/// O(poly(n, m)) typical (Simplex). Maximize `c . x` subject to `a * x <= b`, `x >= 0`; `none` if infeasible or unbounded.
pub fn simplex(a: Mat<F64>, b: List<F64>, c: List<F64>, eps: F64 = 1e-9) -> ?(val: F64, x: List<F64>) = todo

@test
fn matrix_arithmetic_and_pow() = todo

@test
fn exact_elimination_det_rank_solve_inverse() = todo

@test
fn real_elimination_matches_exact() = todo

@test
fn xor_basis_span_kth_and_merge() = todo
