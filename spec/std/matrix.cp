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

    /// A matrix from a 2-D table, multiplied under `over` (default `Semiring(+, *)`):
    /// `Mat(over: Semiring(min, +), d)`. `m * m2`, `pow(m, k)` and a `SegTree(*, ...)` of
    /// matrices all use it. O(rows * cols).
    ///
    /// Row `i` is `R`'s `i`-th position and column `j` is `C`'s `j`-th, so `get(i, j)` reads
    /// `a[R.nth(i), C.nth(j)]`; the matrix has `|R|` rows and `|C|` columns. The keys are not kept.
    ///
    /// cost: O(rows * cols) time and space
    /// empty: a matrix with zero rows or columns
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; Mat([R, C] (r, c) => 10 * Int(r) + Int(c)).get(1, 2) == 12`
    pub fn new<R, C, S>(a: [R, C]T, over: ?S = None) -> Mat<T> = todo

    /// A `rows x cols` matrix with every entry `fill`, multiplied under `over` (default
    /// `Semiring(+, *)`).
    ///
    /// cost: O(rows * cols) time and space
    /// traps: when `rows < 0` or `cols < 0`
    /// example: `Mat<Int>.filled(2, 3, 7).get(1, 2) == 7`
    pub fn filled<S>(rows: Int, cols: Int, fill: T, over: ?S = None) -> Mat<T> = todo

    /// A `rows x cols` matrix of `T.zero`.
    ///
    /// cost: O(rows * cols) time and space
    /// traps: when `rows < 0` or `cols < 0`
    /// example: `Mat<Int>.zeros(2, 3).get(1, 2) == 0`
    pub fn zeros(rows: Int, cols: Int) -> Mat<T> = todo

    /// The `n x n` identity matrix: `T.one` on the diagonal, `T.zero` elsewhere.
    ///
    /// cost: O(n^2) time and space
    /// traps: when `n < 0`
    /// example: `let m = Mat<Int>.identity(3); m.get(1, 1) == 1 and m.get(1, 2) == 0`
    pub fn identity(n: Int) -> Mat<T> = todo

    /// Build a `rows x cols` matrix from a flat row-major list of length `rows * cols`: entry `(r, c)`
    /// is `data[r * cols + c]`.
    ///
    /// cost: O(rows * cols) time and space
    /// empty: a matrix with zero rows or columns when `data` is empty
    /// traps: when `rows < 0`, `cols < 0` or `data.len != rows * cols`
    /// example: `Mat<Int>.from_flat(2, 3, [1, 2, 3, 4, 5, 6]).get(1, 0) == 4`
    pub fn from_flat(rows: Int, cols: Int, data: List<T>) -> Mat<T> = todo

    /// Build a matrix from a list of equal-length rows: `grid[r][c]` is entry `(r, c)`.
    ///
    /// cost: O(rows * cols) time and space
    /// empty: `[]` gives the `0 x 0` matrix
    /// traps: when the rows differ in length
    /// example: `Mat<Int>.from_rows([[1, 2], [3, 4]]).get(1, 0) == 3`
    pub fn from_rows(grid: List<List<T>>) -> Mat<T> = todo

    /// The entry at `(r, c)`, 0-based.
    ///
    /// cost: O(1)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `Mat<Int>.from_rows([[1, 2], [3, 4]]).get(0, 1) == 2`
    pub fn get(self, r: Int, c: Int) -> T = todo

    /// Set the entry at `(r, c)` to `val`.
    ///
    /// cost: O(1)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `var m = Mat<Int>.zeros(2, 2); m.set(1, 0, 5); m.get(1, 0) == 5`
    pub fn set(mut self, r: Int, c: Int, val: T) = todo

    /// The entry at `(r, c)`, lent in place: `m.at(i, j) += x`.
    ///
    /// cost: O(1)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `var m = Mat<Int>.identity(2); m.at(0, 0) += 4; m.get(0, 0) == 5`
    pub fn at(self, r: Int, c: Int) lends mut T:
        lend mut data[r * cols + c]

    /// The transpose, a new `cols x rows` matrix with entry `(c, r)` equal to `self`'s `(r, c)`.
    ///
    /// cost: O(rows * cols) time and space
    /// traps: never
    /// example: `Mat<Int>.from_rows([[1, 2, 3], [4, 5, 6]]).transpose().get(2, 1) == 6`
    pub fn transpose(self) -> Mat<T> = todo

    /// The matrix-vector product `self * v` under the matrix's semiring: entry `r` folds
    /// `get(r, c) * v[c]` over `c` with the semiring's addition.
    ///
    /// cost: O(rows * cols) time, O(rows) space
    /// empty: `rows` copies of the semiring's zero when `cols == 0`
    /// order: indexed by row
    /// traps: when `v.len != cols`
    /// example: `Mat<Int>.from_rows([[1, 2], [3, 4]]).mul_vec([1, 1]) == [3, 7]`
    pub fn mul_vec(self, v: List<T>) -> List<T> = todo

    /// The matrix-multiplication monoid for `n x n` matrices (usable in `SegTree`): identity
    /// `Mat.identity(n)`, op `*`, which is associative but not commutative.
    ///
    /// cost: O(n^2) time and space (for the identity)
    /// traps: when `n < 0`
    /// example: `let m = Mat<Int>.product(2); m.op(m.id, Mat<Int>.from_rows([[1, 2], [3, 4]])).get(1, 1) == 4`
    pub fn product(n: Int) -> Monoid<Mat<T>> = todo

    /// The determinant over an exact field (such as `Mod<P>` for prime `P`), by Gaussian elimination.
    /// The `0 x 0` determinant is `T.one`.
    ///
    /// cost: O(n^3) time, O(n^2) space
    /// traps: when the matrix is not square, or an inverse it needs does not exist (a non-field `T`)
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).det() == Mod<7>(5)`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).det() == Mod<7>(0)`
    pub fn det(self) -> T = todo

    /// The rank over an exact field: the number of pivots elimination finds.
    ///
    /// cost: O(rows * cols * min(rows, cols)) time, O(rows * cols) space
    /// empty: zero
    /// traps: never for a field `T`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).rank() == 1`
    pub fn rank(self) -> Int = todo

    /// Solve `self * x = b` over an exact field. Returns `(particular, nullspace_basis)` or `None`.
    /// Every solution has the form `particular + sum(c_i * basis[i])`. O(rows * cols * min(rows, cols)).
    ///
    /// `particular` is the solution with every free variable zero (what `solve` returns). The basis
    /// has one vector per free (non-pivot) column of the reduced row echelon form, in increasing
    /// column order; the vector for free column `f` has `1` at `f`, `0` at the other free columns,
    /// and whatever the pivot columns need. It is empty when the solution is unique.
    ///
    /// cost: O(rows * cols * min(rows, cols)) time, O(rows * cols) space
    /// empty: with `cols == 0`, `Some(([], []))` when `b` is all zero, else `None`
    /// order: basis vectors by increasing free column
    /// traps: when `b.len != rows`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).solve_space([1, 2]) == Some(([Mod<7>(1), Mod<7>(0)], [[Mod<7>(5), Mod<7>(1)]]))`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).solve_space([1, 0]) == None`
    pub fn solve_space(self, b: List<T>) -> ?(List<T>, List<List<T>>) = todo

    /// Solve `self * x = b` over an exact field. Returns one solution (free variables 0) or `None`.
    ///
    /// The solution is `solve_space(b)`'s particular one: each free (non-pivot) column's variable is
    /// zero.
    ///
    /// cost: O(rows * cols * min(rows, cols)) time, O(rows * cols) space
    /// empty: with `cols == 0`, `Some([])` when `b` is all zero, else `None`
    /// order: indexed by column (variable)
    /// traps: when `b.len != rows`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).solve([5, 6]) == Some([Mod<7>(3), Mod<7>(1)])`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).solve([1, 0]) == None`
    pub fn solve(self, b: List<T>) -> ?List<T> = todo

    /// The multiplicative inverse of a square matrix over an exact field, or `None` if singular, by
    /// Gauss-Jordan elimination.
    ///
    /// cost: O(n^3) time, O(n^2) space
    /// empty: the `0 x 0` matrix is its own inverse
    /// traps: when the matrix is not square
    /// example: `let inv = Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).inverse()!; inv.get(0, 0) == Mod<7>(5) and inv.get(1, 1) == Mod<7>(3)`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).inverse() == None`
    pub fn inverse(self) -> ?Mat<T> = todo

    /// The reduced row echelon form `(rref_mat, rank, pivot_cols)` over a field. Every pivot is
    /// `T.one`, every other entry of a pivot column is zero, the nonzero rows come first, and the
    /// `rank` pivot columns are listed in increasing order. The RREF is unique, so so is the result.
    ///
    /// cost: O(rows * cols * min(rows, cols)) time, O(rows * cols) space
    /// empty: rank zero, no pivots
    /// order: pivot columns increasing
    /// traps: never for a field `T`
    /// example: `let (mat: e, rank: k, pivots: p) = Mat<Mod<7>>.from_rows([[2, 4], [1, 2]]).rref(); k == 1 and p == [0] and e.get(0, 1) == Mod<7>(2) and e.get(1, 1) == Mod<7>(0)`
    pub fn rref(self) -> (mat: Mat<T>, rank: Int, pivots: List<Int>) = todo

    /// The determinant modulo any (possibly composite) `m >= 1`, in `0..m`, by Euclidean (gcd-style)
    /// row elimination, so no inverse is needed. Entries are reduced modulo `m` first, negative ones
    /// included.
    ///
    /// cost: O(n^3 log m)
    /// empty: `1 % m` for the `0 x 0` matrix
    /// traps: when the matrix is not square or `m < 1`
    /// example: `Mat<Int>.from_rows([[1, 2], [3, 4]]).det_mod(6) == 4`
    /// example: `Mat<Int>.from_rows([[2, 0], [0, 3]]).det_mod(6) == 0`
    pub fn det_mod(self, m: Int) -> Int = todo

    /// The characteristic polynomial `det(xI - self)`, lowest degree first, via Hessenberg reduction:
    /// `n + 1` coefficients, the last one `T.one`.
    ///
    /// cost: O(n^3) time, O(n^2) space
    /// empty: `[T.one]` for the `0 x 0` matrix
    /// order: coefficient of `x^0` first
    /// traps: when the matrix is not square
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).char_poly() == [Mod<7>(5), Mod<7>(2), Mod<7>(1)]`
    pub fn char_poly(self) -> List<T> = todo

    /// The Pfaffian of an even-order skew-symmetric matrix over a field, with the sign convention
    /// `pf([[0, a], [-a, 0]]) == a`; `pf(A)^2 == det(A)`. Only the entries above the diagonal are read;
    /// skew-symmetry is not checked.
    ///
    /// cost: O(n^3) time, O(n^2) space
    /// empty: `T.one` for the `0 x 0` matrix
    /// traps: when the matrix is not square or `n` is odd
    /// example: `Mat<Mod<7>>.from_rows([[0, 3], [4, 0]]).pfaffian() == Mod<7>(3)`
    pub fn pfaffian(self) -> T = todo

    /// The adjugate matrix `adj(self)`, the transpose of the cofactor matrix, so
    /// `self * adj(self) == det(self) * I`. Defined even when `rank == n - 1` (and zero when
    /// `rank < n - 1`).
    ///
    /// cost: O(n^3) time, O(n^2) space
    /// empty: the `0 x 0` matrix
    /// traps: when the matrix is not square
    /// example: `let a = Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).adjugate(); a.get(0, 0) == Mod<7>(4) and a.get(0, 1) == Mod<7>(5)`
    /// example: `Mat<Mod<7>>.from_rows([[5]]).adjugate().get(0, 0) == Mod<7>(1)`
    pub fn adjugate(self) -> Mat<T> = todo


impl Add for Mat<T>:
    fn add(a, b) = todo

impl Sub for Mat<T>:
    fn sub(a, b) = todo

impl Mul for Mat<T>:
    fn mul(a, b) = todo

impl Pow for Mat<T>:
    fn pow(a, e: Int) -> Mat<T> = todo

/// The determinant of a floating-point square matrix via partial pivoting with tolerance `eps`.
/// At each column the row with the largest `abs` entry becomes the pivot; if that entry is at most
/// `eps` in absolute value, the determinant is `0.0`.
///
/// cost: O(n^3) time, O(n^2) space
/// empty: `1.0` for the `0 x 0` matrix
/// traps: when `a` is not square
/// example: `abs(det_real(Mat<F64>.from_rows([[2.0, 1.0], [4.0, 3.0]])) - 2.0) < 1e-9`
/// example: `abs(det_real(Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]]))) < 1e-9`
pub fn det_real(a: Mat<F64>, eps: F64 = 1e-9) -> F64 = todo

/// The rank of a floating-point matrix via partial pivoting with tolerance `eps`: a column has a
/// pivot when its largest remaining `abs` entry exceeds `eps`.
///
/// cost: O(rows * cols * min(rows, cols)) time, O(rows * cols) space
/// empty: zero
/// traps: never
/// example: `rank_real(Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]])) == 1`
/// example: `rank_real(Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1e-12]])) == 1`
pub fn rank_real(a: Mat<F64>, eps: F64 = 1e-9) -> Int = todo

/// Solve `a * x = b` over `F64` via partial pivoting with tolerance `eps`. Returns `None` if
/// inconsistent: some row reduces to all-zero coefficients (each at most `eps`) with a right-hand
/// side above `eps` in absolute value. Free variables are `0.0`, as in `Mat.solve`.
///
/// cost: O(rows * cols * min(rows, cols)) time, O(rows * cols) space
/// empty: with `cols == 0`, `Some([])` when every `b` entry is within `eps` of zero, else `None`
/// order: indexed by column (variable)
/// traps: when `b.len != a.rows`
/// example: `let x = solve_real(Mat<F64>.from_rows([[2.0, 1.0], [4.0, 3.0]]), [3.0, 7.0])!; abs(x[0] - 1.0) < 1e-9 and abs(x[1] - 1.0) < 1e-9`
/// example: `solve_real(Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]]), [1.0, 0.0]) == None`
pub fn solve_real(a: Mat<F64>, b: List<F64>, eps: F64 = 1e-9) -> ?List<F64> = todo

/// The inverse of a floating-point square matrix via partial pivoting with tolerance `eps`, or
/// `None` if singular (some pivot is at most `eps` in absolute value).
///
/// cost: O(n^3) time, O(n^2) space
/// empty: the `0 x 0` matrix
/// traps: when `a` is not square
/// example: `let inv = inverse_real(Mat<F64>.from_rows([[2.0, 1.0], [4.0, 3.0]]))!; abs(inv.get(0, 0) - 1.5) < 1e-9 and abs(inv.get(1, 0) + 2.0) < 1e-9`
/// example: `inverse_real(Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]])) == None`
pub fn inverse_real(a: Mat<F64>, eps: F64 = 1e-9) -> ?Mat<F64> = todo

/// Linear basis over GF(2) for non-negative 63-bit integers (`0 <= x < 2^63`).
/// Every operation is O(W) where W <= 63 (`merge` is O(W * rank)).
pub struct XorBasis:
    basis: List<Int>
    size: Int

    /// An empty basis of rank 0, whose span is `{0}`.
    ///
    /// cost: O(W)
    /// traps: never
    /// example: `XorBasis().rank == 0`
    pub fn new() -> XorBasis = todo

    /// The number of linearly independent vectors in the basis: the span has `2^rank` elements.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.add(3); b.rank == 2`
    @get pub fn rank(self) -> Int = todo

    /// Insert `x >= 0` into the basis. Returns `true` iff the rank increased, that is, iff `x` was not
    /// already in the span (`0` never is new). The basis is kept fully reduced: no vector has another
    /// vector's highest bit set.
    ///
    /// cost: O(W)
    /// traps: when `x < 0`
    /// example: `var b = XorBasis(); let p = b.add(6); let q = b.add(5); let r = b.add(3); p and q and not r`
    /// example: `not XorBasis().add(0)`
    pub fn add(mut self, x: Int) -> Bool = todo

    /// Reduce `x >= 0` against the basis: the minimum of `x ^ s` over `s` in the span, which is 0 iff
    /// `x` is in the span.
    ///
    /// cost: O(W)
    /// traps: when `x < 0`
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.reduce(4) == 1 and b.reduce(3) == 0`
    pub fn reduce(self, x: Int) -> Int = todo

    /// Whether `x >= 0` is a XOR-combination of basis vectors: `reduce(x) == 0`.
    ///
    /// cost: O(W)
    /// traps: when `x < 0`
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.contains(3) and not b.contains(4)`
    pub fn contains(self, x: Int) -> Bool = todo

    /// The maximum value of `init ^ s` over all `s` in the span.
    ///
    /// cost: O(W)
    /// ties: none possible; the maximum value is unique
    /// traps: when `init < 0`
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.max_xor() == 6 and b.max_xor(1) == 7`
    pub fn max_xor(self, init: Int = 0) -> Int = todo

    /// The minimum value of `init ^ s` over all `s` in the span: the same as `reduce(init)`, and 0 for
    /// the default `init`.
    ///
    /// cost: O(W)
    /// ties: none possible; the minimum value is unique
    /// traps: when `init < 0`
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.min_xor(4) == 1 and b.min_xor() == 0`
    pub fn min_xor(self, init: Int = 0) -> Int = todo

    /// The 0-indexed `k`-th smallest value in the span, or `None` if `k < 0` or `k >= 2^rank`. `kth(0)`
    /// is 0, and `kth` over `0..2^rank` is strictly increasing and lists the whole span.
    ///
    /// cost: O(W^2)
    /// empty: only `kth(0) == Some(0)` for an empty basis
    /// ties: none possible; span values are distinct
    /// traps: never
    /// example: `var b = XorBasis(); b.add(6); b.add(5); [b.kth(k) for k in 0..5] == [Some(0), Some(3), Some(5), Some(6), None]`
    pub fn kth(self, k: Int) -> ?Int = todo

    /// Merge another basis into `self`, so `self` spans the union of both spans; `o` is unchanged.
    ///
    /// cost: O(W * o.rank)
    /// traps: never
    /// example: `var a = XorBasis(); a.add(6); var b = XorBasis(); b.add(8); a.merge(b); a.rank == 2 and a.contains(14)`
    pub fn merge(mut self, o: XorBasis) = todo

    /// The monoid under basis merge (usable in `SegTree`): identity the empty basis, op the merged
    /// basis. Commutative and idempotent on spans.
    ///
    /// cost: O(W) (for the identity)
    /// traps: never
    /// example: `let m = XorBasis.monoid(); m.id.rank == 0`
    pub fn monoid() -> Monoid<XorBasis> = todo

    /// The independent basis vectors currently stored, fully reduced (no vector has another's highest
    /// bit set), so the list depends only on the span. One vector per rank.
    ///
    /// cost: O(W) time and space
    /// empty: `[]`
    /// order: decreasing (by highest set bit)
    /// traps: never
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.basis_vecs() == [5, 3]`
    pub fn basis_vecs(self) -> List<Int> = todo

    /// A new basis spanning the intersection of the two GF(2) vector subspaces.
    ///
    /// cost: O(W^2)
    /// traps: never
    /// example: `var a = XorBasis(); a.add(6); a.add(5); var b = XorBasis(); b.add(3); b.add(8); let c = a.intersect(b); c.rank == 1 and c.contains(3)`
    pub fn intersect(self, o: XorBasis) -> XorBasis = todo

fn merge_xor(a: XorBasis, b: XorBasis) -> XorBasis = todo

/// O(n^3 log max_exp) build, O(n^2 log e) query. Precomputed binary powers of a square matrix
/// for fast multi-query `A^e * v`.
pub struct MatPow<T>:
    n: Int

    /// Precompute `a^(2^k)` for every `k` with `2^k <= max_exp`, multiplied under `a`'s semiring.
    ///
    /// cost: O(n^3 log max_exp) time, O(n^2 log max_exp) space
    /// traps: when `a` is not square or `max_exp < 0`
    /// example: `let p = MatPow(Mat<Int>.from_rows([[1, 1], [1, 0]]), 10); p.apply_pow(5, [1, 0]) == [8, 5]`
    pub fn new(a: Mat<T>, max_exp: Int) -> MatPow<T> = todo

    /// Compute `a^e * v` for `0 <= e <= max_exp`, as matrix-vector products with the stored powers
    /// for the set bits of `e`. `e == 0` returns `v`.
    ///
    /// cost: O(n^2 log e) time, O(n) space
    /// empty: `v` itself when `e == 0`
    /// order: indexed by row
    /// traps: when `e` is not in `0..=max_exp` or `v.len != n`
    /// example: `let p = MatPow(Mat<Int>.from_rows([[1, 1], [1, 0]]), 10); p.apply_pow(0, [1, 0]) == [1, 0]`
    pub fn apply_pow(self, e: Int, v: List<T>) -> List<T> = todo

/// O(n * m / 64) storage. Bit-packed dense matrix over GF(2) with word-parallel elimination.
pub struct BitMat(rows: Int, cols: Int):
    /// A zero GF(2) matrix of `rows x cols`, one bit per entry.
    ///
    /// cost: O(rows * cols / 64) time and space
    /// traps: when `rows < 0` or `cols < 0`
    /// example: `BitMat(2, 3).rank() == 0`
    pub fn new(rows: Int, cols: Int) -> BitMat = todo

    /// The `n x n` GF(2) identity matrix.
    ///
    /// cost: O(n^2 / 64) time and space
    /// traps: when `n < 0`
    /// example: `BitMat.identity(3).get(1, 1) and not BitMat.identity(3).get(1, 2)`
    pub fn identity(n: Int) -> BitMat = todo

    /// Read bit `(r, c)`.
    ///
    /// cost: O(1)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `var m = BitMat(2, 2); m.set(0, 1, true); m.get(0, 1) and not m.get(1, 0)`
    pub fn get(self, r: Int, c: Int) -> Bool = todo

    /// Write bit `(r, c)`.
    ///
    /// cost: O(1)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `var m = BitMat(2, 2); m.set(0, 1, true); m.set(0, 1, false); not m.get(0, 1)`
    pub fn set(mut self, r: Int, c: Int, v: Bool) = todo

    /// The determinant over GF(2): `true` iff the matrix is invertible. The `0 x 0` determinant is
    /// `true`.
    ///
    /// cost: O(n^3 / 64)
    /// traps: when the matrix is not square
    /// example: `var m = BitMat.identity(2); m.set(0, 1, true); m.det()`
    /// example: `var m = BitMat(2, 2); m.set(0, 0, true); m.set(1, 0, true); not m.det()`
    pub fn det(self) -> Bool = todo

    /// The rank over GF(2).
    ///
    /// cost: O(rows * cols * min(rows, cols) / 64)
    /// empty: zero
    /// traps: never
    /// example: `var m = BitMat(2, 2); m.set(0, 0, true); m.set(0, 1, true); m.set(1, 0, true); m.set(1, 1, true); m.rank() == 1`
    pub fn rank(self) -> Int = todo

    /// Any solution to `self * x == b` over GF(2), or `None`: the one with every free (non-pivot)
    /// variable `false`, as in `Mat.solve`.
    ///
    /// cost: O(rows * cols * min(rows, cols) / 64)
    /// empty: with `cols == 0`, `Some([])` when `b` is all `false`, else `None`
    /// order: indexed by column (variable)
    /// traps: when `b.len != rows`
    /// example: `var m = BitMat.identity(2); m.set(0, 1, true); m.solve([true, true]) == Some([false, true])`
    /// example: `var m = BitMat(2, 2); m.set(0, 0, true); m.set(0, 1, true); m.set(1, 0, true); m.set(1, 1, true); m.solve([true, false]) == None`
    pub fn solve(self, b: List<Bool>) -> ?List<Bool> = todo

    /// One solution `x0` and a nullspace basis over GF(2), or `None`. `x0` is `solve(b)`'s solution;
    /// the basis has one vector per free column, in increasing column order, with `true` at that
    /// column and `false` at the other free columns, as in `Mat.solve_space`.
    ///
    /// cost: O(rows * cols * min(rows, cols) / 64)
    /// empty: with `cols == 0`, `Some(([], []))` when `b` is all `false`, else `None`
    /// order: basis vectors by increasing free column
    /// traps: when `b.len != rows`
    /// example: `var m = BitMat(2, 2); m.set(0, 0, true); m.set(0, 1, true); m.set(1, 0, true); m.set(1, 1, true); m.solve_space([true, true]) == Some((x0: [true, false], basis: [[true, true]]))`
    pub fn solve_space(self, b: List<Bool>) -> ?(x0: List<Bool>, basis: List<List<Bool>>) = todo

    /// The inverse of a square GF(2) matrix, or `None` when it is singular.
    ///
    /// cost: O(n^3 / 64)
    /// empty: the `0 x 0` matrix
    /// traps: when the matrix is not square
    /// example: `var m = BitMat.identity(2); m.set(0, 1, true); let inv = m.inverse()!; inv.get(0, 1) and inv.get(1, 1) and not inv.get(1, 0)`
    pub fn inverse(self) -> ?BitMat = todo

    /// The matrix product `self * o` over GF(2) (AND for multiply, XOR for add).
    ///
    /// cost: O(rows * cols * o.cols / 64)
    /// traps: when `self.cols != o.rows`
    /// example: `var m = BitMat.identity(2); m.set(0, 1, true); let p = m.mul(m); not p.get(0, 1) and p.get(0, 0)`
    pub fn mul(self, o: BitMat) -> BitMat = todo

/// O(62) per push and query. Online prefix linear basis answering subarray XOR queries over any range `r`.
pub struct PrefixXorBasis:
    len: Int

    /// An empty prefix XOR basis, with no positions pushed.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `PrefixXorBasis().rank(0..0) == 0`
    pub fn new() -> PrefixXorBasis = todo

    /// Append `x` at the next position (`0`, `1`, ...), keeping for each bit the basis vector with the
    /// rightmost source index, so any subarray's span reads off the vectors whose index lies in it.
    ///
    /// cost: O(62) time and space
    /// traps: when `x < 0` or `x >= 2^62`
    /// example: `var p = PrefixXorBasis(); p.push(6); p.push(5); p.rank(0..2) == 2`
    pub fn push(mut self, x: Int) = todo

    /// The maximum `init ^ s` for `s` in the span of the values at positions `r`.
    ///
    /// cost: O(62)
    /// empty: `init` (the span of nothing is `{0}`)
    /// ties: none possible; the maximum value is unique
    /// traps: when `r` is non-empty and reaches outside `0..len`, or `init < 0`
    /// example: `var p = PrefixXorBasis(); p.push(6); p.push(5); p.push(3); p.max_xor(1..3) == 6 and p.max_xor(2..3) == 3`
    pub fn max_xor(self, r: Range<Int>, init: Int = 0) -> Int = todo

    /// Whether `x` is in the span of the values at positions `r`.
    ///
    /// cost: O(62)
    /// empty: `true` only for `x == 0`
    /// traps: when `r` is non-empty and reaches outside `0..len`, or `x < 0`
    /// example: `var p = PrefixXorBasis(); p.push(6); p.push(5); p.push(3); p.contains(1..3, 6) and not p.contains(1..2, 3)`
    pub fn contains(self, r: Range<Int>, x: Int) -> Bool = todo

    /// The rank (dimension) of the span of the values at positions `r`.
    ///
    /// cost: O(62)
    /// empty: zero
    /// traps: when `r` is non-empty and reaches outside `0..len`
    /// example: `var p = PrefixXorBasis(); p.push(6); p.push(5); p.push(3); p.rank(0..3) == 2 and p.rank(2..3) == 1`
    pub fn rank(self, r: Range<Int>) -> Int = todo

/// The determinant of a sparse `n x n` matrix via Wiedemann's algorithm. `entries` lists
/// `(row, col, value)`; repeated cells add up, and missing cells are zero. Randomized with a fixed
/// seed and retried until the minimal polynomial is certified, so the answer is always exact and
/// deterministic; only the running time is an expectation. `P` must be prime.
///
/// cost: O(n * (entries.len + n)) expected time, O(n + entries.len) space
/// empty: `Mod<P>(1)` for `n == 0`
/// traps: when `n < 0` or a row or column is not in `0..n`
/// example: `let e: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(2)), (1, 1, Mod(3))]; sparse_det(2, e) == Mod<998244353>(6)`
/// example: `let e: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(1)), (0, 1, Mod(1))]; sparse_det(2, e) == Mod<998244353>(0)`
pub fn sparse_det<P>(n: Int, entries: List<(Int, Int, Mod<P>)>) -> Mod<P> = todo

/// Solve `A x = b` for a sparse `n x n` matrix via Wiedemann's algorithm, with `entries` as in
/// `sparse_det`. Returns the unique solution, or `None` when `A` is singular (even if `b` happens
/// to be in its range). Deterministic, like `sparse_det`.
///
/// cost: O(n * (entries.len + n)) expected time, O(n + entries.len) space
/// empty: `Some([])` for `n == 0`
/// order: indexed by column (variable)
/// traps: when `n < 0`, `b.len != n`, or a row or column is not in `0..n`
/// example: `let e: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(2)), (1, 1, Mod(3))]; sparse_solve(2, e, [Mod(4), Mod(9)]) == Some([Mod<998244353>(2), Mod<998244353>(3)])`
/// example: `let e: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(1))]; sparse_solve(2, e, [Mod(1), Mod(0)]) == None`
pub fn sparse_solve<P>(n: Int, entries: List<(Int, Int, Mod<P>)>, b: List<Mod<P>>) -> ?List<Mod<P>> = todo

/// Maximize `c . x` subject to `a * x <= b`, `x >= 0`; `None` if infeasible or unbounded.
///
/// Two-phase simplex with Bland's rule, comparing against `eps`. When the optimum is reached at
/// several points, `x` is the vertex the pivoting reaches; `val` is the optimum either way.
///
/// cost: O(poly(n, m)) typical (exponential in the worst case); O(rows * cols) space
/// empty: with no variables, `Some((val: 0.0, x: []))` when every `b` entry is at least `-eps`
/// order: `x` indexed by column (variable)
/// traps: when `b.len != a.rows` or `c.len != a.cols`
/// example: `let r = simplex(Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1.0]]), [1.0, 2.0], [1.0, 1.0])!; abs(r.val - 3.0) < 1e-9 and abs(r.x[1] - 2.0) < 1e-9`
/// example: `simplex(Mat<F64>.from_rows([[1.0]]), [-1.0], [1.0]) == None`
/// example: `simplex(Mat<F64>.from_rows([[-1.0]]), [0.0], [1.0]) == None`
pub fn simplex(a: Mat<F64>, b: List<F64>, c: List<F64>, eps: F64 = 1e-9) -> ?(val: F64, x: List<F64>) = todo

@test
fn matrix_arithmetic_and_pow() = todo

@test
fn exact_elimination_det_rank_solve_inverse() = todo

@test
fn real_elimination_matches_exact() = todo

@test
fn xor_basis_span_kth_and_merge() = todo
