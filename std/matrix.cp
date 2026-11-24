//! Dense matrices, exact and floating-point Gaussian elimination, and GF(2) XOR bases.
//!
//! `Mat<T>` stores an `rows x cols` matrix in flat row-major order (`data[r * cols + c]`).
//! Exact elimination (`det`, `rank`, `solve`, `solve_space`, `inverse`) works over any field
//! type with `T.zero`, `T.one`, exact `==`, and `/` (such as `Mod<P>` for prime `P`).
//! Over an ordered `T` such as `F64` elimination pivots on the largest `abs` entry and treats an
//! entry within the `eps:` argument of zero as zero.
use std.algebra.{Monoid, Semiring}
use std.math.{mul_mod, bit}
use std.mod.{Mod}
use std.rand.{Rng}
use std.seq.{collect}

const SEED = 998_244_353
const RAND_HI = 2_000_000_000

/// A dense `rows x cols` matrix stored flat in row-major order.
pub struct Mat<T>:
    pub rows: Int
    pub cols: Int
    pub data: List<T>
    ring: ?Ring<T> = None

    /// A matrix from a 2-D table, multiplied under `over` (default `Semiring(+, *)`):
    /// `Mat(over: Semiring(min, +), d)`. `m * m2`, `m ** k` and a `SegTree(*, ...)` of
    /// matrices all use it. \(O(\mathit{rows} \cdot \mathit{cols})\).
    ///
    /// Row `i` is `R`'s `i`-th position and column `j` is `C`'s `j`-th, so `get(i, j)` reads
    /// `a[R.nth(i), C.nth(j)]`; the matrix has `|R|` rows and `|C|` columns. The keys are not kept.
    ///
    /// Under any other `over:` the entries are `?W`, `None` being the semiring's zero (no path):
    /// `Mat(over: Semiring(min, +), d)` with `d: [V, V]?Int` multiplies shortest walks.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols})\) time and space
    /// empty: a matrix with zero rows or columns
    /// traps: never
    ///
    /// example: `domain R = 0..2; domain C = 0..3; Mat([R, C] (r, c) => 10 * Int(r) + Int(c)).get(1, 2) == 12`
    /// example: `domain V = 0..2; let m = Mat([V, V] (i, j) => if i == j: None else: Some(Int(i) + 1), over: Semiring(min, +)); (m * m).get(0, 0) == Some(3)`
    pub fn new<R, C, S>(a: [R, C]T, over: S = Semiring(+, *)) -> Mat<T>:
        var d = List<T>()
        for r in R:
            for c in C: d.push(a[r, c])
        Mat(R.count, C.count, d, ring_of(over))

    /// A matrix from a list of equal-length rows: `Mat<Z>.of(adj)` for an adjacency table read
    /// as nested lists. Row `i` is `rows[i]`.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols})\) time and space
    /// empty: a `0 x 0` matrix
    /// traps: when the rows differ in length
    /// example: `let m = Mat<Int>.of([[1, 2, 3], [4, 5, 6]]); m.get(1, 2) == 6 and m.get(0, 1) == 2`
    pub fn of(rows: List<List<T>>) -> Mat<T>:
        let cols = if rows.is_empty: 0 else: rows[0].len
        var d = List<T>()
        for row in rows:
            assert row.len == cols, "Mat.of requires rows of equal length"
            for x in row: d.push(x)
        Mat(rows.len, cols, d)

    /// A `rows x cols` matrix with every entry `fill`, multiplied under `over` (default
    /// `Semiring(+, *)`).
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols})\) time and space
    /// traps: when `rows < 0` or `cols < 0`
    /// example: `Mat<Int>.filled(2, 3, 7).get(1, 2) == 7`
    /// example: `Mat<?Int>.filled(2, 2, None, over: Semiring(min, +)).get(0, 1) == None`
    pub fn filled<S>(rows: Int, cols: Int, fill: T, over: S = Semiring(+, *)) -> Mat<T>:
        assert rows >= 0 and cols >= 0, "Mat.filled requires rows >= 0 and cols >= 0"
        Mat(rows, cols, List.filled(rows * cols, fill), ring_of(over))

    /// A `rows x cols` matrix of `T.zero`.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols})\) time and space
    /// traps: when `rows < 0` or `cols < 0`
    /// example: `Mat<Int>.zeros(2, 3).get(1, 2) == 0`
    pub fn zeros(rows: Int, cols: Int) -> Mat<T>:
        assert rows >= 0 and cols >= 0
        Mat(rows, cols, List.filled(rows * cols, T.zero))

    /// The `n x n` identity matrix: `T.one` on the diagonal, `T.zero` elsewhere.
    ///
    /// cost: \(O(n^2)\) time and space
    /// traps: when `n < 0`
    /// example: `let m = Mat<Int>.identity(3); m.get(1, 1) == 1 and m.get(1, 2) == 0`
    pub fn identity(n: Int) -> Mat<T>:
        assert n >= 0
        var d = List.filled(n * n, T.zero)
        for i in 0..n: d[i * n + i] = T.one
        Mat(n, n, d)

    /// Build a `rows x cols` matrix from a flat row-major list of length `rows * cols`: entry `(r, c)`
    /// is `data[r * cols + c]`.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols})\) time and space
    /// empty: a matrix with zero rows or columns when `data` is empty
    /// traps: when `rows < 0`, `cols < 0` or `data.len != rows * cols`
    /// example: `Mat<Int>.from_flat(2, 3, [1, 2, 3, 4, 5, 6]).get(1, 0) == 4`
    pub fn from_flat(rows: Int, cols: Int, data: List<T>) -> Mat<T>:
        assert rows >= 0 and cols >= 0 and data.len == rows * cols
        Mat(rows, cols, data.clone())

    fn unit(n: Int, r: ?Ring<T>) -> Mat<T>:
        if T: Add:
            if let g = r: return Mat<T>.eye_in(n, g)
            Mat<T>.identity(n)
        else:
            Mat<T>.eye_in(n, r!)

    fn eye_in(n: Int, g: Ring<T>) -> Mat<T>:
        var d = List.filled(n * n, g.zero)
        for i in 0..n: d[i * n + i] = g.one
        Mat(n, n, d, Some(g))

    /// Build a matrix from a list of equal-length rows: `grid[r][c]` is entry `(r, c)`.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols})\) time and space
    /// empty: `[]` gives the `0 x 0` matrix
    /// traps: when the rows differ in length
    /// example: `Mat<Int>.from_rows([[1, 2], [3, 4]]).get(1, 0) == 3`
    pub fn from_rows(grid: List<List<T>>) -> Mat<T>:
        let r = grid.len
        if r == 0: return Mat(0, 0, List<T>())
        let c = grid[0].len
        var d = List<T>()
        for row in grid:
            assert row.len == c, "all matrix rows must have equal length"
            for x in row: d.push(x)
        Mat(r, c, d)

    /// The entry at `(r, c)`, 0-based.
    ///
    /// cost: \(O(1)\)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `Mat<Int>.from_rows([[1, 2], [3, 4]]).get(0, 1) == 2`
    /// trap: `Mat<Int>.zeros(2, 2).get(0, 2)`
    /// trap: `Mat<Int>.zeros(2, 2).get(-1, 0)`
    pub fn get(self, r: Int, c: Int) -> T = data[self.cell(r, c)]

    /// Set the entry at `(r, c)` to `val`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `var m = Mat<Int>.zeros(2, 2); m.set(1, 0, 5); m.get(1, 0) == 5`
    /// trap: `var m = Mat<Int>.zeros(2, 2); m.set(2, 0, 5)`
    pub fn set(mut self, r: Int, c: Int, val: T):
        data[self.cell(r, c)] = val

    /// The entry at `(r, c)`, lent in place: `m.at(i, j) += x`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `var m = Mat<Int>.identity(2); m.at(0, 0) += 4; m.get(0, 0) == 5`
    /// trap: `var m = Mat<Int>.zeros(2, 2); m.at(0, 2) += 1`
    pub fn at(self, r: Int, c: Int) -> lends mut T:
        lend mut data[self.cell(r, c)]

    /// The flat offset of `(r, c)`, trapping unless `r` is in `0..rows` and `c` in `0..cols`:
    /// every accessor goes through it, so no row spills into the next.
    fn cell(self, r: Int, c: Int) -> Int:
        assert 0 <= r < rows, "row {r} is outside 0..{rows}"
        assert 0 <= c < cols, "column {c} is outside 0..{cols}"
        r * cols + c

    /// The transpose, a new `cols x rows` matrix with entry `(c, r)` equal to `self`'s `(r, c)`.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols})\) time and space
    /// traps: never
    /// example: `Mat<Int>.from_rows([[1, 2, 3], [4, 5, 6]]).transpose().get(2, 1) == 6`
    pub fn transpose(self) -> Mat<T>:
        var d = List.filled(rows * cols, T.zero)
        for i in 0..rows:
            for j in 0..cols: d[j * rows + i] = data[i * cols + j]
        Mat(cols, rows, d)

    /// The matrix-vector product `self * v` under the matrix's semiring: entry `r` folds
    /// `get(r, c) * v[c]` over `c` with the semiring's addition.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols})\) time, \(O(\mathit{rows})\) space
    /// empty: `rows` copies of the semiring's zero when `cols == 0`
    /// order: indexed by row
    /// traps: when `v.len != cols`
    /// example: `Mat<Int>.from_rows([[1, 2], [3, 4]]).mul_vec([1, 1]) == [3, 7]`
    pub fn mul_vec(self, v: List<T>) -> List<T>:
        assert v.len == cols, "vector length must match matrix columns"
        if T: Add:
            if T: Mul:
                if ring == None:
                    var out = List.filled(rows, T.zero)
                    for i in 0..rows:
                        var acc = T.zero
                        let off = i * cols
                        for j in 0..cols: acc = acc + data[off + j] * v[j]
                        out[i] = acc
                    return out
        let g = ring!
        var out = List.filled(rows, g.zero)
        for i in 0..rows:
            for j in 0..cols: out[i] = g.add(out[i], g.mul(data[i * cols + j], v[j]))
        out

    /// The matrix-multiplication monoid for `n x n` matrices under `over` (usable in `SegTree`):
    /// identity the semiring's identity matrix, op `*`, which is associative but not commutative.
    ///
    /// cost: \(O(n^2)\) time and space (for the identity)
    /// traps: when `n < 0`
    /// example: `let m = Mat<Int>.product(2); m.op(m.id, Mat<Int>.from_rows([[1, 2], [3, 4]])).get(1, 1) == 4`
    /// example: `let m = Mat<?Int>.product(2, over: Semiring(min, +)); m.id.get(0, 0) == Some(0) and m.id.get(0, 1) == None`
    pub fn product<S>(n: Int, over: S = Semiring(+, *)) -> Monoid<Mat<T>>:
        assert n >= 0
        let r: ?Ring<T> = ring_of(over)
        Monoid(Mat<T>.unit(n, r), (x, y) => x * y)

    fn mag(x: T) -> T = if x < T.zero: T.zero - x else: x

    fn negligible(x: T, eps: T) -> Bool:
        if T: Ord:
            return Mat<T>.mag(x) <= eps
        else:
            return x == T.zero

    /// The pivot row for `col` among rows `lo..hi` of the flat table `m` with row length `stride`, or
    /// `hi` if there is none: the largest `abs` entry above `eps` for an ordered `T`, else the first
    /// nonzero.
    fn pivot(m: List<T>, lo: Int, hi: Int, stride: Int, col: Int, eps: T) -> Int:
        if T: Ord:
            var piv = lo
            for r in lo + 1..hi:
                if Mat<T>.mag(m[r * stride + col]) > Mat<T>.mag(m[piv * stride + col]): piv = r
            return if lo < hi and Mat<T>.mag(m[piv * stride + col]) > eps: piv else: hi
        else:
            var piv = lo
            while piv < hi and m[piv * stride + col] == T.zero: piv += 1
            return piv

    /// The determinant over a field, by Gaussian elimination. The `0 x 0` determinant is `T.one`.
    ///
    /// Over an ordered `T` such as `F64` the pivot is the largest `abs` entry of its column and an
    /// entry within `eps` of zero counts as zero; over `Mod<P>` the pivot is the first nonzero entry.
    ///
    /// cost: \(O(n^3)\) time, \(O(n^2)\) space
    /// traps: when the matrix is not square, or an inverse it needs does not exist (a non-field `T`)
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).det() == Mod<7>(5)`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).det() == Mod<7>(0)`
    /// example: `abs(Mat<F64>.from_rows([[2.0, 1.0], [4.0, 3.0]]).det(eps: 1e-9) - 2.0) < 1e-9`
    pub fn det(self, eps: ?T = None) -> T:
        assert rows == cols, "det requires a square matrix"
        let n = rows
        var m = data.collect()
        var ans = T.one
        let e = eps ?? T.zero
        for col in 0..n:
            let piv = Mat<T>.pivot(m, col, n, n, col, e)
            if piv == n: return T.zero
            if piv != col:
                for c in col..n: m.swap(col * n + c, piv * n + c)
                ans = T.zero - ans
            let lead = m[col * n + col]
            ans = ans * lead
            let inv_l = T.one / lead
            for r in col + 1..n:
                let f = m[r * n + col]
                if Mat<T>.negligible(f, e): continue
                let scale = f * inv_l
                for c in col..n: m[r * n + c] -= scale * m[col * n + c]
        ans

    /// The rank over a field: the number of pivots elimination finds, pivoting as `det` does.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols} \cdot \min(\mathit{rows}, \mathit{cols}))\) time, \(O(\mathit{rows} \cdot \mathit{cols})\) space
    /// empty: zero
    /// traps: never for a field `T`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).rank() == 1`
    /// example: `Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1e-12]]).rank(eps: 1e-9) == 1`
    pub fn rank(self, eps: ?T = None) -> Int:
        let e = eps ?? T.zero
        var m = data.collect()
        var r = 0
        for col in 0..cols:
            if r == rows: break
            let piv = Mat<T>.pivot(m, r, rows, cols, col, e)
            if piv == rows: continue
            if piv != r:
                for c in col..cols: m.swap(r * cols + c, piv * cols + c)
            let inv_l = T.one / m[r * cols + col]
            for i in r + 1..rows:
                let f = m[i * cols + col]
                if Mat<T>.negligible(f, e): continue
                let scale = f * inv_l
                for c in col..cols: m[i * cols + c] -= scale * m[r * cols + c]
            r += 1
        r

    /// Solve `self * x = b` over a field. Returns `(particular, nullspace_basis)` or `None`.
    /// Every solution has the form `particular + sum(c_i * basis[i])`. \(O(\mathit{rows} \cdot \mathit{cols} \cdot \min(\mathit{rows}, \mathit{cols}))\).
    ///
    /// `particular` is the solution with every free variable zero (what `solve` returns). The basis
    /// has one vector per free (non-pivot) column of the reduced row echelon form, in increasing
    /// column order; the vector for free column `f` has `1` at `f`, `0` at the other free columns,
    /// and whatever the pivot columns need. It is empty when the solution is unique.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols} \cdot \min(\mathit{rows}, \mathit{cols}))\) time, \(O(\mathit{rows} \cdot \mathit{cols})\) space
    /// empty: with `cols == 0`, `Some(([], []))` when `b` is all zero, else `None`
    /// order: basis vectors by increasing free column
    /// traps: when `b.len != rows`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).solve_space([1, 2]) == Some(([Mod<7>(1), Mod<7>(0)], [[Mod<7>(5), Mod<7>(1)]]))`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).solve_space([1, 0]) == None`
    pub fn solve_space(self, b: List<T>, eps: ?T = None) -> ?(List<T>, List<List<T>>):
        assert b.len == rows
        let e = eps ?? T.zero
        let k = cols
        let stride = k + 1
        var aug = List.filled(rows * stride, T.zero)
        for i in 0..rows:
            for j in 0..k: aug[i * stride + j] = data[i * k + j]
            aug[i * stride + k] = b[i]
        var pivot_row = List.filled(k, -1)
        var r = 0
        for col in 0..k:
            if r == rows: break
            let piv = Mat<T>.pivot(aug, r, rows, stride, col, e)
            if piv == rows: continue
            if piv != r:
                for c in col..=k: aug.swap(r * stride + c, piv * stride + c)
            let inv_l = T.one / aug[r * stride + col]
            for c in col..=k: aug[r * stride + c] = aug[r * stride + c] * inv_l
            for i in 0..rows:
                if i == r: continue
                let f = aug[i * stride + col]
                if Mat<T>.negligible(f, e): continue
                for c in col..=k: aug[i * stride + c] = aug[i * stride + c] - f * aug[r * stride + c]
            pivot_row[col] = r
            r += 1
        for i in r..rows:
            if not Mat<T>.negligible(aug[i * stride + k], e): return None
        var part = List.filled(k, T.zero)
        var basis = List<List<T>>()
        for col in 0..k:
            if pivot_row[col] != -1:
                part[col] = aug[pivot_row[col] * stride + k]
            else:
                var v = List.filled(k, T.zero)
                v[col] = T.one
                for pcol in 0..k:
                    if pivot_row[pcol] != -1: v[pcol] = T.zero - aug[pivot_row[pcol] * stride + col]
                basis.push(v)
        (part, basis)

    /// Solve `self * x = b` over a field. Returns one solution (free variables 0) or `None`.
    ///
    /// The solution is `solve_space(b)`'s particular one: each free (non-pivot) column's variable is
    /// zero.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols} \cdot \min(\mathit{rows}, \mathit{cols}))\) time, \(O(\mathit{rows} \cdot \mathit{cols})\) space
    /// empty: with `cols == 0`, `Some([])` when `b` is all zero, else `None`
    /// order: indexed by column (variable)
    /// traps: when `b.len != rows`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).solve([5, 6]) == Some([Mod<7>(3), Mod<7>(1)])`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).solve([1, 0]) == None`
    pub fn solve(self, b: List<T>, eps: ?T = None) -> ?List<T>:
        let res = self.solve_space(b, eps)
        if res == None: return None
        res!.0

    /// The multiplicative inverse of a square matrix over a field, or `None` if singular, by
    /// Gauss-Jordan elimination.
    ///
    /// cost: \(O(n^3)\) time, \(O(n^2)\) space
    /// empty: the `0 x 0` matrix is its own inverse
    /// traps: when the matrix is not square
    /// example: `let inv = Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).inverse()!; inv.get(0, 0) == Mod<7>(5) and inv.get(1, 1) == Mod<7>(3)`
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [2, 4]]).inverse() == None`
    pub fn inverse(self, eps: ?T = None) -> ?Mat<T>:
        assert rows == cols, "inverse requires a square matrix"
        let n = rows
        let stride = 2 * n
        let e = eps ?? T.zero
        var aug = List.filled(n * stride, T.zero)
        for i in 0..n:
            for j in 0..n: aug[i * stride + j] = data[i * n + j]
            aug[i * stride + n + i] = T.one
        for col in 0..n:
            let piv = Mat<T>.pivot(aug, col, n, stride, col, e)
            if piv == n: return None
            if piv != col:
                for c in col..stride: aug.swap(col * stride + c, piv * stride + c)
            let inv_l = T.one / aug[col * stride + col]
            for c in col..stride: aug[col * stride + c] = aug[col * stride + c] * inv_l
            for i in 0..n:
                if i == col: continue
                let f = aug[i * stride + col]
                if Mat<T>.negligible(f, e): continue
                for c in col..stride: aug[i * stride + c] = aug[i * stride + c] - f * aug[col * stride + c]
        var out = List.filled(n * n, T.zero)
        for i in 0..n:
            for j in 0..n: out[i * n + j] = aug[i * stride + n + j]
        Mat(n, n, out)

    /// The reduced row echelon form `(rref_mat, rank, pivot_cols)` over a field.
    ///
    /// Every pivot is `T.one`, every other entry of a pivot column is zero, the nonzero rows come
    /// first, and the `rank` pivot columns are listed in increasing order. The RREF is unique, so
    /// so is the result.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols} \cdot \min(\mathit{rows}, \mathit{cols}))\) time, \(O(\mathit{rows} \cdot \mathit{cols})\) space
    /// empty: rank zero, no pivots
    /// order: pivot columns increasing
    /// traps: never for a field `T`
    /// example: `let (mat: e, rank: k, pivots: p) = Mat<Mod<7>>.from_rows([[2, 4], [1, 2]]).rref(); k == 1 and p == [0] and e.get(0, 1) == Mod<7>(2) and e.get(1, 1) == Mod<7>(0)`
    pub fn rref(self) -> (mat: Mat<T>, rank: Int, pivots: List<Int>):
        var m = data.collect()
        var pivots = List<Int>()
        var r = 0
        for col in 0..cols:
            if r == rows: break
            var piv = r
            while piv < rows and m[piv * cols + col] == T.zero: piv += 1
            if piv == rows: continue
            if piv != r:
                for c in col..cols: m.swap(r * cols + c, piv * cols + c)
            let inv_l = T.one / m[r * cols + col]
            for c in col..cols: m[r * cols + c] *= inv_l
            for i in 0..rows:
                if i == r: continue
                let f = m[i * cols + col]
                if f == T.zero: continue
                for c in col..cols: m[i * cols + c] -= f * m[r * cols + c]
            pivots.push(col)
            r += 1
        (mat: Mat(rows, cols, m), rank: r, pivots:)

    /// The determinant modulo any (possibly composite) `m >= 1`, in `0..m`, by Euclidean (gcd-style)
    /// row elimination, so no inverse is needed. Entries are reduced modulo `m` first, negative ones
    /// included.
    ///
    /// cost: \(O(n^3 \log m)\)
    /// empty: `1 % m` for the `0 x 0` matrix
    /// traps: when the matrix is not square or `m < 1`
    /// example: `Mat<Int>.from_rows([[1, 2], [3, 4]]).det_mod(6) == 4`
    /// example: `Mat<Int>.from_rows([[2, 0], [0, 3]]).det_mod(6) == 0`
    pub fn det_mod(self, m: Int) -> Int:
        assert rows == cols, "det_mod requires a square matrix"
        assert m >= 1, "det_mod requires m >= 1"
        let n = rows
        var a = [Int(x) % m for x in data]
        var ans = 1 % m
        for col in 0..n:
            for r in col + 1..n:
                while a[r * n + col] != 0:
                    let q = a[col * n + col] / a[r * n + col]
                    if q != 0:
                        for c in col..n: a[col * n + c] = (
                            a[col * n + c] - mul_mod(q, a[r * n + c], m)
                        ) % m
                    for c in col..n: a.swap(col * n + c, r * n + c)
                    ans = (m - ans) % m
            if a[col * n + col] == 0: return 0
            ans = mul_mod(ans, a[col * n + col], m)
        ans

    /// The characteristic polynomial `det(xI - self)`, lowest degree first, via Hessenberg reduction:
    /// `n + 1` coefficients, the last one `T.one`.
    ///
    /// cost: \(O(n^3)\) time, \(O(n^2)\) space
    /// empty: `[T.one]` for the `0 x 0` matrix
    /// order: coefficient of `x^0` first
    /// traps: when the matrix is not square
    /// example: `Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).char_poly() == [Mod<7>(5), Mod<7>(2), Mod<7>(1)]`
    pub fn char_poly(self) -> List<T>:
        assert rows == cols, "char_poly requires a square matrix"
        let n = rows
        var h = data.collect()
        for col in 0..n - 2:
            var piv = col + 1
            while piv < n and h[piv * n + col] == T.zero: piv += 1
            if piv == n: continue
            if piv != col + 1:
                for c in 0..n: h.swap(piv * n + c, (col + 1) * n + c)
                for r in 0..n: h.swap(r * n + piv, r * n + col + 1)
            let inv_l = T.one / h[(col + 1) * n + col]
            for r in col + 2..n:
                let f = h[r * n + col] * inv_l
                if f == T.zero: continue
                for c in 0..n: h[r * n + c] -= f * h[(col + 1) * n + c]
                for i in 0..n: h[i * n + col + 1] += f * h[i * n + r]
        var ps = List<List<T>>()
        ps.push([T.one])
        for k in 0..n:
            var next = List.filled(k + 2, T.zero)
            let pk = ps[k]
            for i in 0..k + 1:
                next[i + 1] = next[i + 1] + pk[i]
                next[i] = next[i] - h[k * n + k] * pk[i]
            var t = T.one
            for i in (0..k).rev:
                t = t * h[(i + 1) * n + i]
                let coef = h[i * n + k] * t
                if coef == T.zero: continue
                let pi = ps[i]
                for j in 0..i + 1: next[j] = next[j] - coef * pi[j]
            ps.push(next)
        ps[n]

    /// The Pfaffian of an even-order skew-symmetric matrix over a field, with the sign convention
    /// `pf([[0, a], [-a, 0]]) == a`; `pf(A)^2 == det(A)`. Only the entries above the diagonal are read;
    /// skew-symmetry is not checked.
    ///
    /// cost: \(O(n^3)\) time, \(O(n^2)\) space
    /// empty: `T.one` for the `0 x 0` matrix
    /// traps: when the matrix is not square or `n` is odd
    /// example: `Mat<Mod<7>>.from_rows([[0, 3], [4, 0]]).pfaffian() == Mod<7>(3)`
    pub fn pfaffian(self) -> T:
        assert rows == cols, "pfaffian requires a square matrix"
        let n = rows
        assert n % 2 == 0, "pfaffian requires an even order"
        var a = List.filled(n * n, T.zero)
        for i in 0..n:
            for j in i + 1..n:
                a[i * n + j] = data[i * n + j]
                a[j * n + i] = T.zero - data[i * n + j]
        var res = T.one
        var k = 0
        while k < n:
            var piv = k + 1
            while piv < n and a[k * n + piv] == T.zero: piv += 1
            if piv == n: return T.zero
            if piv != k + 1:
                for c in 0..n: a.swap(piv * n + c, (k + 1) * n + c)
                for r in 0..n: a.swap(r * n + piv, r * n + k + 1)
                res = T.zero - res
            let lead = a[k * n + k + 1]
            res = res * lead
            let inv_l = T.one / lead
            for i in k + 2..n:
                let f = a[k * n + i] * inv_l
                if f != T.zero:
                    for r in 0..n: a[r * n + i] = a[r * n + i] - f * a[r * n + k + 1]
                    for c in 0..n: a[i * n + c] = a[i * n + c] - f * a[(k + 1) * n + c]
                let g = T.zero - a[(k + 1) * n + i] * inv_l
                if g != T.zero:
                    for r in 0..n: a[r * n + i] = a[r * n + i] - g * a[r * n + k]
                    for c in 0..n: a[i * n + c] = a[i * n + c] - g * a[k * n + c]
            k += 2
        res

    /// The adjugate matrix `adj(self)`, the transpose of the cofactor matrix, so
    /// `self * adj(self) == det(self) * I`. Defined even when `rank == n - 1` (and zero when
    /// `rank < n - 1`).
    ///
    /// cost: \(O(n^3)\) time, \(O(n^2)\) space
    /// empty: the `0 x 0` matrix
    /// traps: when the matrix is not square
    /// example: `let a = Mat<Mod<7>>.from_rows([[1, 2], [3, 4]]).adjugate(); a.get(0, 0) == Mod<7>(4) and a.get(0, 1) == Mod<7>(5)`
    /// example: `Mat<Mod<7>>.from_rows([[5]]).adjugate().get(0, 0) == Mod<7>(1)`
    pub fn adjugate(self) -> Mat<T> where T: Mul + Div:
        assert rows == cols, "adjugate requires a square matrix"
        let n = rows
        if n == 0: return Mat(0, 0, List<T>())
        if n == 1: return Mat(1, 1, [T.one])
        let k = self.rank()
        if k == n:
            let d = self.det()
            let inv = self.inverse()!
            return Mat(n, n, [d * x for x in inv.data])
        if k < n - 1: return Mat<T>.zeros(n, n)
        let x = self.solve_space(List.filled(n, T.zero))!.1[0]
        let y = self.transpose().solve_space(List.filled(n, T.zero))!.1[0]
        var r = 0
        while x[r] == T.zero: r += 1
        var c = 0
        while y[c] == T.zero: c += 1
        var minor = List<T>()
        for i in 0..n:
            if i == c: continue
            for j in 0..n:
                if j != r: minor.push(data[i * n + j])
        var cof = Mat(n - 1, n - 1, minor).det()
        if (r + c) % 2 == 1: cof = T.zero - cof
        let lam = cof / (x[r] * y[c])
        var out = List.filled(n * n, T.zero)
        for i in 0..n:
            for j in 0..n: out[i * n + j] = lam * x[i] * y[j]
        Mat(n, n, out)

/// Two matrices are equal when their shapes and entries are; the semiring is not compared.
impl Eq for Mat<T>:
    fn eq(a, b) = a.rows == b.rows and a.cols == b.cols and a.data == b.data

impl Add for Mat<T>:
    fn add(a, b):
        assert a.rows == b.rows and a.cols == b.cols
        Mat(a.rows, a.cols, [a.data[i] + b.data[i] for i in 0..a.data.len])

impl Sub for Mat<T>:
    fn sub(a, b):
        assert a.rows == b.rows and a.cols == b.cols
        Mat(a.rows, a.cols, [a.data[i] - b.data[i] for i in 0..a.data.len])

impl Mul for Mat<T>:
    fn mul(a, b):
        assert a.cols == b.rows, "matrix dimensions do not match for multiplication"
        let (r, m, c) = (a.rows, a.cols, b.cols)
        if T: Add:
            if T: Mul:
                if a.ring == None:
                    var out = List.filled(r * c, T.zero)
                    unchecked:
                        for i in 0..r:
                            let ai = i * m
                            let oi = i * c
                            for k in 0..m:
                                let f = a.data[ai + k]
                                let bk = k * c
                                for j in 0..c: out[oi + j] = out[oi + j] + f * b.data[bk + j]
                    return Mat(r, c, out)
        let g = a.ring!
        let (add, mul) = (g.add, g.mul)
        var acc = List.filled(r * c, g.zero)
        for i in 0..r:
            for k in 0..m:
                let f = a.data[i * m + k]
                if f == g.zero: continue
                for j in 0..c: acc[i * c + j] = add(acc[i * c + j], mul(f, b.data[k * c + j]))
        Mat(r, c, acc, a.ring)

impl Pow for Mat<T>:
    fn pow(a, e: Int) -> Mat<T>:
        assert a.rows == a.cols and e >= 0
        var base = a
        var acc = Mat<T>.unit(a.rows, a.ring)
        var k = e
        while k > 0:
            if (k & 1) == 1: acc *= base
            k >>= 1
            if k > 0: base = base * base
        acc

/// The `(zero, one, add, mul)` a non-default `Mat(over:)` multiplies under. On `?W` entries the
/// semiring over `W` is lifted so `None` is the zero, absorbing under `mul`.
struct Ring<T>(zero: T, one: T, add: (T, T) -> T, mul: (T, T) -> T)

fn same<A, B>(a: A, b: B) -> Bool = a == b

fn ring_of<T, S>(over: S) -> ?Ring<T>:
    if same(over.add, +) and same(over.mul, *): return None
    if T: Add:
        let add = Monoid<T>.of(over.add)
        let mul = Monoid<T>.of(over.mul)
        Some(Ring(add.id, mul.id, add.op, mul.op))
    else:
        Some(lift_ring(over))

fn lift_ring<W, S>(over: S) -> Ring<?W>:
    let add = Monoid<W>.of(over.add)
    let mul = Monoid<W>.of(over.mul)
    Ring(
        None,
        Some(mul.id),
        (a: ?W, b: ?W) => if let x = a: (if let y = b: Some(add.op(x, y)) else: a) else: b,
        (a: ?W, b: ?W) => if let x = a: (if let y = b: Some(mul.op(x, y)) else: None) else: None,
    )

/// Linear basis over GF(2) for non-negative 63-bit integers (`0 <= x < 2^63`).
/// Every operation is \(O(W)\) where W <= 63 (`merge` is \(O(W \cdot \mathit{rank})\)).
pub struct XorBasis:
    basis: List<Int>
    size: Int

    /// An empty basis of rank 0, whose span is `{0}`.
    ///
    /// cost: \(O(W)\)
    /// traps: never
    /// example: `XorBasis().rank == 0`
    pub fn new() -> XorBasis = XorBasis(List.filled(63, 0), 0)

    /// The number of linearly independent vectors in the basis: the span has `2^rank` elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.add(3); b.rank == 2`
    @get pub fn rank(self) -> Int = size

    /// Insert `x >= 0` into the basis. Returns `true` iff the rank increased, that is, iff `x` was not
    /// already in the span (`0` never is new). The basis is kept fully reduced: no vector has another
    /// vector's highest bit set.
    ///
    /// cost: \(O(W)\)
    /// traps: when `x < 0`
    /// example: `var b = XorBasis(); let p = b.add(6); let q = b.add(5); let r = b.add(3); p and q and not r`
    /// example: `var b = XorBasis(); not b.add(0)`
    pub fn add(mut self, x: Int) -> Bool:
        assert x >= 0
        var v = x
        var b = 62
        while b >= 0 and v > 0:
            if v.bit(b):
                if basis[b] == 0:
                    basis[b] = v
                    size += 1
                    return true
                v ^= basis[b]
            b -= 1
        false

    /// Reduce `x >= 0` against the basis: the minimum of `x ^ s` over `s` in the span, which is 0 iff
    /// `x` is in the span.
    ///
    /// cost: \(O(W)\)
    /// traps: when `x < 0`
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.reduce(4) == 1 and b.reduce(3) == 0`
    pub fn reduce(self, x: Int) -> Int:
        assert x >= 0
        var v = x
        var b = 62
        while b >= 0 and v > 0:
            if v.bit(b) and basis[b] != 0: v ^= basis[b]
            b -= 1
        v

    /// Whether `x >= 0` is a XOR-combination of basis vectors: `reduce(x) == 0`.
    ///
    /// cost: \(O(W)\)
    /// traps: when `x < 0`
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.contains(3) and not b.contains(4)`
    pub fn contains(self, x: Int) -> Bool = self.reduce(x) == 0

    /// The maximum value of `init ^ s` over all `s` in the span.
    ///
    /// cost: \(O(W)\)
    /// ties: None possible; the maximum value is unique
    /// traps: when `init < 0`
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.max_xor() == 6 and b.max_xor(1) == 7`
    pub fn max_xor(self, init: Int = 0) -> Int:
        assert init >= 0
        var v = init
        var b = 62
        while b >= 0:
            if basis[b] != 0 and (v ^ basis[b]) > v: v ^= basis[b]
            b -= 1
        v

    /// The minimum value of `init ^ s` over all `s` in the span: the same as `reduce(init)`, and 0 for
    /// the default `init`.
    ///
    /// cost: \(O(W)\)
    /// ties: None possible; the minimum value is unique
    /// traps: when `init < 0`
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.min_xor(4) == 1 and b.min_xor() == 0`
    pub fn min_xor(self, init: Int = 0) -> Int:
        assert init >= 0
        var v = init
        var b = 62
        while b >= 0:
            if basis[b] != 0 and (v ^ basis[b]) < v: v ^= basis[b]
            b -= 1
        v

    /// The 0-indexed `k`-th smallest value in the span, or `None` if `k < 0` or `k >= 2^rank`. `kth(0)`
    /// is 0, and `kth` over `0..2^rank` is strictly increasing and lists the whole span.
    ///
    /// cost: \(O(W^2)\)
    /// empty: only `kth(0) == Some(0)` for an empty basis
    /// ties: None possible; span values are distinct
    /// traps: never
    /// example: `var b = XorBasis(); b.add(6); b.add(5); [b.kth(k) for k in 0..5] == [Some(0), Some(3), Some(5), Some(6), None]`
    pub fn kth(self, k: Int) -> ?Int:
        if k < 0: return None
        if size < 63 and k >= (1 << size): return None
        // Build reduced row echelon form (lowest bit first)
        var rref = List<Int>()
        for b in 0..63:
            if basis[b] == 0: continue
            var v = basis[b]
            for prev in rref:
                if (v ^ prev) < v: v ^= prev
            rref.push(v)
        var ans = 0
        for i in 0..rref.len:
            if k.bit(i): ans ^= rref[i]
        ans

    /// Merge another basis into `self`, so `self` spans the union of both spans; `o` is unchanged.
    ///
    /// cost: \(O(W \cdot o.\mathit{rank})\)
    /// traps: never
    /// example: `var a = XorBasis(); a.add(6); var b = XorBasis(); b.add(8); a.merge(b); a.rank == 2 and a.contains(14)`
    pub fn merge(mut self, o: XorBasis):
        for b in 0..63:
            if o.basis[b] != 0:
                let _ = self.add(o.basis[b])

    /// The monoid under basis merge (usable in `SegTree`): identity the empty basis, op the merged
    /// basis. Commutative and idempotent on spans.
    ///
    /// cost: \(O(W)\) (for the identity)
    /// traps: never
    /// example: `let m = XorBasis.monoid(); m.id.rank == 0`
    pub fn monoid() -> Monoid<XorBasis> = Monoid(XorBasis.new(), merge_xor)

    /// The independent basis vectors currently stored, fully reduced (no vector has another's highest
    /// bit set), so the list depends only on the span. One vector per rank.
    ///
    /// cost: \(O(W)\) time and space
    /// empty: `[]`
    /// order: decreasing (by highest set bit)
    /// traps: never
    /// example: `var b = XorBasis(); b.add(6); b.add(5); b.basis_vecs() == [5, 3]`
    pub fn basis_vecs(self) -> List<Int>:
        var red = List.filled(63, 0)
        for b in 0..63:
            var v = basis[b]
            if v == 0: continue
            for c in (0..b).rev:
                if red[c] != 0 and v.bit(c): v ^= red[c]
            red[b] = v
            for d in b + 1..63:
                if red[d] != 0 and red[d].bit(b): red[d] ^= v
        [red[b] for b in (0..63).rev if red[b] != 0]

    /// A new basis spanning the intersection of the two GF(2) vector subspaces.
    ///
    /// cost: \(O(W^2)\)
    /// traps: never
    /// example: `var a = XorBasis(); a.add(6); a.add(5); var b = XorBasis(); b.add(3); b.add(8); let c = a.intersect(b); c.rank == 1 and c.contains(3)`
    pub fn intersect(self, o: XorBasis) -> XorBasis:
        var bx = List.filled(63, 0)
        var by = List.filled(63, 0)
        for b in 0..63:
            if basis[b] != 0:
                bx[b] = basis[b]
                by[b] = basis[b]
        var out = XorBasis.new()
        for b in 0..63:
            if o.basis[b] == 0: continue
            var x = o.basis[b]
            var y = 0
            var bit = 62
            while bit >= 0 and x != 0:
                if x.bit(bit):
                    if bx[bit] == 0:
                        bx[bit] = x
                        by[bit] = y
                        x = 0
                        y = -1
                    else:
                        x ^= bx[bit]
                        y ^= by[bit]
                bit -= 1
            if y > 0: let _ = out.add(y)
        out

fn merge_xor(a: XorBasis, b: XorBasis) -> XorBasis:
    var out = XorBasis.new()
    out.merge(a)
    out.merge(b)
    out

/// \(O(n^3 \log \mathit{max\_exp})\) build, \(O(n^2 \log e)\) query. Precomputed binary powers of a square matrix
/// for fast multi-query `A^e * v`.
pub struct MatPow<T>:
    pub n: Int
    _pw: List<Mat<T>>

    /// Precompute `a^(2^k)` for every `k` with `2^k <= max_exp`, multiplied under `a`'s semiring.
    ///
    /// cost: \(O(n^3 \log \mathit{max\_exp})\) time, \(O(n^2 \log \mathit{max\_exp})\) space
    /// traps: when `a` is not square or `max_exp < 0`
    /// example: `let p = MatPow(Mat<Int>.from_rows([[1, 1], [1, 0]]), 10); p.apply_pow(5, [1, 0]) == [8, 5]`
    pub fn new(a: Mat<T>, max_exp: Int) -> MatPow<T>:
        assert a.rows == a.cols and max_exp >= 0
        var pw = List<Mat<T>>()
        var k = 1
        while k <= max_exp:
            pw.push(if pw.len == 0: copy(a) else: pw.last! * pw.last!)
            k *= 2
        MatPow(a.rows, pw)

    /// Compute `a^e * v` for `0 <= e <= max_exp`, as matrix-vector products with the stored powers
    /// for the set bits of `e`. `e == 0` returns `v`.
    ///
    /// cost: \(O(n^2 \log e)\) time, \(O(n)\) space
    /// empty: `v` itself when `e == 0`
    /// order: indexed by row
    /// traps: when `e` is not in `0..=max_exp` or `v.len != n`
    /// example: `let p = MatPow(Mat<Int>.from_rows([[1, 1], [1, 0]]), 10); p.apply_pow(0, [1, 0]) == [1, 0]`
    pub fn apply_pow(self, e: Int, v: List<T>) -> List<T>:
        assert 0 <= e and (e == 0 or e < (1 << _pw.len))
        var out = v
        for k in 0.._pw.len:
            if e.bit(k): out = _pw[k].mul_vec(out)
        out

/// \(O(n \cdot m / 64)\) storage. Bit-packed dense matrix over GF(2) with word-parallel elimination.
pub struct BitMat(pub rows: Int, pub cols: Int):
    bits: List<Int>

    /// A zero GF(2) matrix of `rows x cols`, one bit per entry.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols} / 64)\) time and space
    /// traps: when `rows < 0` or `cols < 0`
    /// example: `BitMat(2, 3).rank() == 0`
    pub fn new(rows: Int, cols: Int) -> BitMat:
        assert rows >= 0 and cols >= 0
        BitMat(rows, cols, List.filled(rows * bit_words(cols), 0))

    /// The `n x n` GF(2) identity matrix.
    ///
    /// cost: \(O(n^2 / 64)\) time and space
    /// traps: when `n < 0`
    /// example: `BitMat.identity(3).get(1, 1) and not BitMat.identity(3).get(1, 2)`
    pub fn identity(n: Int) -> BitMat:
        var m = BitMat.new(n, n)
        for i in 0..n: m.set(i, i, true)
        m

    /// Read bit `(r, c)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `var m = BitMat(2, 2); m.set(0, 1, true); m.get(0, 1) and not m.get(1, 0)`
    /// trap: `BitMat(2, 2).get(0, 2)`
    pub fn get(self, r: Int, c: Int) -> Bool:
        assert 0 <= r and r < rows and 0 <= c and c < cols
        bits[r * bit_words(cols) + c / 62].bit((c % 62))

    /// Write bit `(r, c)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `r` is not in `0..rows` or `c` is not in `0..cols`
    /// example: `var m = BitMat(2, 2); m.set(0, 1, true); m.set(0, 1, false); not m.get(0, 1)`
    pub fn set(mut self, r: Int, c: Int, v: Bool):
        assert 0 <= r and r < rows and 0 <= c and c < cols
        let k = r * bit_words(cols) + c / 62
        let bit = 1 << (c % 62)
        if v: bits[k] |= bit
        else: bits[k] &= ~bit

    /// The determinant over GF(2): `true` iff the matrix is invertible. The `0 x 0` determinant is
    /// `true`.
    ///
    /// cost: \(O(n^3 / 64)\)
    /// traps: when the matrix is not square
    /// example: `var m = BitMat.identity(2); m.set(0, 1, true); m.det()`
    /// example: `var m = BitMat(2, 2); m.set(0, 0, true); m.set(1, 0, true); not m.det()`
    pub fn det(self) -> Bool:
        assert rows == cols, "det requires a square matrix"
        self.rank() == rows

    /// The rank over GF(2).
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols} \cdot \min(\mathit{rows}, \mathit{cols}) / 64)\)
    /// empty: zero
    /// traps: never
    /// example: `var m = BitMat(2, 2); m.set(0, 0, true); m.set(0, 1, true); m.set(1, 0, true); m.set(1, 1, true); m.rank() == 1`
    pub fn rank(self) -> Int:
        var a = bits.collect()
        bit_rref(rows, cols, mut a).len

    /// Any solution to `self * x == b` over GF(2), or `None`: the one with every free (non-pivot)
    /// variable `false`, as in `Mat.solve`.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols} \cdot \min(\mathit{rows}, \mathit{cols}) / 64)\)
    /// empty: with `cols == 0`, `Some([])` when `b` is all `false`, else `None`
    /// order: indexed by column (variable)
    /// traps: when `b.len != rows`
    /// example: `var m = BitMat.identity(2); m.set(0, 1, true); m.solve([true, true]) == Some([false, true])`
    /// example: `var m = BitMat(2, 2); m.set(0, 0, true); m.set(0, 1, true); m.set(1, 0, true); m.set(1, 1, true); m.solve([true, false]) == None`
    pub fn solve(self, b: List<Bool>) -> ?List<Bool>:
        if let sp = self.solve_space(b): return Some(sp.x0)
        None

    /// One solution `x0` and a nullspace basis over GF(2), or `None`.
    ///
    /// `x0` is `solve(b)`'s solution; the basis has one vector per free column, in increasing
    /// column order, with `true` at that column and `false` at the other free columns, as in
    /// `Mat.solve_space`.
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols} \cdot \min(\mathit{rows}, \mathit{cols}) / 64)\)
    /// empty: with `cols == 0`, `Some(([], []))` when `b` is all `false`, else `None`
    /// order: basis vectors by increasing free column
    /// traps: when `b.len != rows`
    /// example: `var m = BitMat(2, 2); m.set(0, 0, true); m.set(0, 1, true); m.set(1, 0, true); m.set(1, 1, true); m.solve_space([true, true]) == Some((x0: [true, false], basis: [[true, true]]))`
    pub fn solve_space(self, b: List<Bool>) -> ?(x0: List<Bool>, basis: List<List<Bool>>):
        assert b.len == rows, "solve_space needs one right-hand side per row"
        var aug = BitMat.new(rows, cols + 1)
        for r in 0..rows:
            for c in 0..cols:
                if self.get(r, c): aug.set(r, c, true)
            if b[r]: aug.set(r, cols, true)
        let piv = bit_rref(rows, cols + 1, mut aug.bits)
        if piv.len > 0 and piv.last! == cols: return None
        var x0 = List.filled(cols, false)
        var is_piv = List.filled(cols, false)
        for i in 0..piv.len:
            x0[piv[i]] = aug.get(i, cols)
            is_piv[piv[i]] = true
        var basis = List<List<Bool>>()
        for f in 0..cols:
            if is_piv[f]: continue
            var v = List.filled(cols, false)
            v[f] = true
            for i in 0..piv.len: v[piv[i]] = aug.get(i, f)
            basis.push(v)
        Some((x0:, basis:))

    /// The inverse of a square GF(2) matrix, or `None` when it is singular.
    ///
    /// cost: \(O(n^3 / 64)\)
    /// empty: the `0 x 0` matrix
    /// traps: when the matrix is not square
    /// example: `var m = BitMat.identity(2); m.set(0, 1, true); let inv = m.inverse()!; inv.get(0, 1) and inv.get(1, 1) and not inv.get(1, 0)`
    pub fn inverse(self) -> ?BitMat:
        assert rows == cols, "inverse requires a square matrix"
        let n = rows
        var aug = BitMat.new(n, 2 * n)
        for r in 0..n:
            for c in 0..n:
                if self.get(r, c): aug.set(r, c, true)
            aug.set(r, n + r, true)
        let piv = bit_rref(n, 2 * n, mut aug.bits)
        if 0 < n <= piv[n - 1]: return None
        var out = BitMat.new(n, n)
        for r in 0..n:
            for c in 0..n:
                if aug.get(r, n + c): out.set(r, c, true)
        Some(out)

    /// The matrix product `self * o` over GF(2) (AND for multiply, XOR for add).
    ///
    /// cost: \(O(\mathit{rows} \cdot \mathit{cols} \cdot o.\mathit{cols} / 64)\)
    /// traps: when `self.cols != o.rows`
    /// example: `var m = BitMat.identity(2); m.set(0, 1, true); let p = m.mul(m); not p.get(0, 1) and p.get(0, 0)`
    pub fn mul(self, o: BitMat) -> BitMat:
        assert cols == o.rows, "mul needs `self.cols == o.rows`"
        var out = BitMat.new(rows, o.cols)
        let w = bit_words(o.cols)
        for i in 0..rows:
            for t in 0..cols:
                if self.get(i, t):
                    for k in 0..w: out.bits[i * w + k] ^= o.bits[t * w + k]
        out

fn bit_words(cols: Int) -> Int = (cols + 61) / 62

fn bit_rref(rows: Int, cols: Int, mut a: List<Int>) -> List<Int>:
    let w = bit_words(cols)
    var piv = List<Int>()
    var r = 0
    for c in 0..cols:
        if r == rows: break
        let k = c / 62
        let bit = 1 << (c % 62)
        var p = r
        while p < rows and (a[p * w + k] & bit) == 0: p += 1
        if p == rows: continue
        if p != r:
            for j in 0..w: a.swap(p * w + j, r * w + j)
        for i in 0..rows:
            if i != r and (a[i * w + k] & bit) != 0:
                for j in 0..w: a[i * w + j] ^= a[r * w + j]
        piv.push(c)
        r += 1
    piv

/// \(O(62)\) per push and query. Online prefix linear basis answering subarray XOR queries over any range `r`.
pub struct PrefixXorBasis:
    pub len: Int
    vecs: List<Int>
    from: List<Int>

    /// An empty prefix XOR basis, with no positions pushed.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `PrefixXorBasis().rank(0..0) == 0`
    pub fn new() -> PrefixXorBasis = PrefixXorBasis(0, List.filled(63, 0), List.filled(63, -1))

    /// Append `x` at the next position (`0`, `1`, ...), keeping for each bit the basis vector with the
    /// rightmost source index, so any subarray's span reads off the vectors whose index lies in it.
    ///
    /// cost: \(O(62)\) time and space
    /// traps: when `x < 0` or `x >= 2^62`
    /// example: `var p = PrefixXorBasis(); p.push(6); p.push(5); p.rank(0..2) == 2`
    pub fn push(mut self, x: Int):
        assert x >= 0, "`push` needs `x >= 0`"
        let base = len * 63
        for b in 0..63:
            vecs.push(vecs[base + b])
            from.push(from[base + b])
        len += 1
        let top = len * 63
        var v = x
        var at = len - 1
        for b in (0..63).rev:
            if not v.bit(b): continue
            if vecs[top + b] == 0:
                vecs[top + b] = v
                from[top + b] = at
                return
            if from[top + b] < at:
                let tv = vecs[top + b]
                let ta = from[top + b]
                vecs[top + b] = v
                from[top + b] = at
                v = tv
                at = ta
            v ^= vecs[top + b]

    /// The maximum `init ^ s` for `s` in the span of the values at positions `r`.
    ///
    /// cost: \(O(62)\)
    /// empty: `init` (the span of nothing is `{0}`)
    /// ties: None possible; the maximum value is unique
    /// traps: when `r` is non-empty and reaches outside `0..len`, or `init < 0`
    /// example: `var p = PrefixXorBasis(); p.push(6); p.push(5); p.push(3); p.max_xor(1..3) == 6 and p.max_xor(2..3) == 3`
    pub fn max_xor(self, r: Range<Int>, init: Int = 0) -> Int:
        if r.is_empty: return init
        let l = r.first!
        let top = (l + r.len) * 63
        var res = init
        for b in (0..63).rev:
            if from[top + b] >= l and (res ^ vecs[top + b]) > res: res ^= vecs[top + b]
        res

    /// Whether `x` is in the span of the values at positions `r`.
    ///
    /// cost: \(O(62)\)
    /// empty: `true` only for `x == 0`
    /// traps: when `r` is non-empty and reaches outside `0..len`, or `x < 0`
    /// example: `var p = PrefixXorBasis(); p.push(6); p.push(5); p.push(3); p.contains(1..3, 6) and not p.contains(1..2, 3)`
    pub fn contains(self, r: Range<Int>, x: Int) -> Bool:
        if r.is_empty: return x == 0
        let l = r.first!
        let top = (l + r.len) * 63
        var v = x
        for b in (0..63).rev:
            if v.bit(b) and from[top + b] >= l: v ^= vecs[top + b]
        v == 0

    /// The rank (dimension) of the span of the values at positions `r`.
    ///
    /// cost: \(O(62)\)
    /// empty: zero
    /// traps: when `r` is non-empty and reaches outside `0..len`
    /// example: `var p = PrefixXorBasis(); p.push(6); p.push(5); p.push(3); p.rank(0..3) == 2 and p.rank(2..3) == 1`
    pub fn rank(self, r: Range<Int>) -> Int:
        if r.is_empty: return 0
        let l = r.first!
        let top = (l + r.len) * 63
        let k = count(from[top + b] >= l for b in 0..63)
        k

/// The determinant of a sparse `n x n` matrix via Wiedemann's algorithm.
///
/// `entries` lists `(row, col, value)`; repeated cells add up, and missing cells are zero.
/// Randomized with a fixed seed and retried until the minimal polynomial is certified, so the
/// answer is always exact and deterministic; only the running time is an expectation. `P` must be
/// prime.
///
/// cost: \(O(n \cdot (\mathit{entries}.\mathit{len} + n))\) expected time, \(O(n + \mathit{entries}.\mathit{len})\) space
/// empty: `Mod<P>(1)` for `n == 0`
/// traps: when `n < 0` or a row or column is not in `0..n`
/// example: `let e: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(2)), (1, 1, Mod(3))]; sparse_det(2, e) == Mod<998244353>(6)`
/// example: `let e: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(1)), (0, 1, Mod(1))]; sparse_det(2, e) == Mod<998244353>(0)`
pub fn sparse_det<P>(n: Int, entries: List<(Int, Int, Mod<P>)>) -> Mod<P>:
    assert n >= 0, "sparse_det requires n >= 0"
    for (
        r,
        c,
        _,
    ) in entries: assert 0 <= r and r < n and 0 <= c and c < n, "sparse_det entry outside 0..n"
    if n == 0: return Mod<P>(1)
    var rng = Rng(SEED)
    loop:
        let w = wiedemann(n, entries, mut rng)
        if w.singular: return Mod<P>(0)
        if w.f.len == n + 1:
            var det = w.f[0]
            if n % 2 == 1: det *= Mod<P>(-1)
            for d in w.diag: det /= d
            return det

/// Solve `A x = b` for a sparse `n x n` matrix via Wiedemann's algorithm, with `entries` as in
/// `sparse_det`.
///
/// Returns the unique solution, or `None` when `A` is singular (even if `b` happens to be in its
/// range). Deterministic, like `sparse_det`.
///
/// cost: \(O(n \cdot (\mathit{entries}.\mathit{len} + n))\) expected time, \(O(n + \mathit{entries}.\mathit{len})\) space
/// empty: `Some([])` for `n == 0`
/// order: indexed by column (variable)
/// traps: when `n < 0`, `b.len != n`, or a row or column is not in `0..n`
/// example: `let e: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(2)), (1, 1, Mod(3))]; sparse_solve(2, e, [Mod(4), Mod(9)]) == Some([Mod<998244353>(2), Mod<998244353>(3)])`
/// example: `let e: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(1))]; sparse_solve(2, e, [Mod(1), Mod(0)]) == None`
pub fn sparse_solve<P>(n: Int, entries: List<(Int, Int, Mod<P>)>, b: List<Mod<P>>) -> ?List<Mod<P>>:
    assert n >= 0, "sparse_solve requires n >= 0"
    assert b.len == n, "sparse_solve requires b.len == n"
    for (
        r,
        c,
        _,
    ) in entries: assert 0 <= r and r < n and 0 <= c and c < n, "sparse_solve entry outside 0..n"
    if n == 0: return Some([])
    var rng = Rng(SEED)
    loop:
        let w = wiedemann(n, entries, mut rng)
        if w.singular: return None
        if w.f.len == n + 1:
            var y = List.filled(n, Mod<P>(0))
            var t = b
            for i in 1..=n:
                for k in 0..n: y[k] += w.f[i] * t[k]
                if i < n: t = sparse_apply(n, entries, w.diag, t)
            let scale = Mod<P>(-1) / w.f[0]
            let x = [y[k] * scale * w.diag[k] for k in 0..n]
            let ax = sparse_apply(n, entries, [Mod<P>(1) for _ in 0..n], x)
            let ok = (0..n).all(k => ax[k] == b[k])
            if ok: return Some(x)

fn sparse_apply<P>(
    n: Int,
    entries: List<(Int, Int, Mod<P>)>,
    diag: List<Mod<P>>,
    v: List<Mod<P>>,
) -> List<Mod<P>>:
    var out = List.filled(n, Mod<P>(0))
    for (r, c, a) in entries: out[r] += a * diag[c] * v[c]
    out

fn wiedemann<P>(
    n: Int,
    entries: List<(Int, Int, Mod<P>)>,
    mut rng: Rng,
) -> (f: List<Mod<P>>, diag: List<Mod<P>>, singular: Bool):
    var diag = [Mod<P>(rng.int(1, RAND_HI)) for _ in 0..n]
    for k in 0..n:
        if diag[k] == Mod<P>(0): diag[k] = Mod<P>(1)
    let u = [Mod<P>(rng.int(0, RAND_HI)) for _ in 0..n]
    var v = [Mod<P>(rng.int(0, RAND_HI)) for _ in 0..n]
    var seq: List<Mod<P>> = []
    for _ in 0..2 * n:
        var dot = Mod<P>(0)
        for k in 0..n: dot += u[k] * v[k]
        seq.push(dot)
        v = sparse_apply(n, entries, diag, v)
    let c = berlekamp_massey(seq)
    let deg = c.len - 1
    var f = List.filled(deg + 1, Mod<P>(0))
    for i in 0..=deg: f[deg - i] = c[i]
    (f:, diag:, singular: deg >= 1 and f[0] == Mod<P>(0))

fn berlekamp_massey<P>(s: List<Mod<P>>) -> List<Mod<P>>:
    var cur: List<Mod<P>> = [Mod<P>(1)]
    var prev: List<Mod<P>> = [Mod<P>(1)]
    var len = 0
    var shift = 1
    var last = Mod<P>(1)
    for i in 0..s.len:
        var d = s[i]
        for j in 1..=min(len, cur.len - 1): d += cur[j] * s[i - j]
        if d == Mod<P>(0):
            shift += 1
            continue
        let coef = d / last
        var next = cur.clone()
        while next.len < prev.len + shift: next.push(Mod<P>(0))
        for j in 0..prev.len: next[j + shift] -= coef * prev[j]
        if 2 * len <= i:
            len = i + 1 - len
            prev = cur
            last = d
            shift = 1
        else:
            shift += 1
        cur = next
    while cur.len < len + 1: cur.push(Mod<P>(0))
    [cur[i] for i in 0..=len]

/// Maximize `c . x` subject to `a * x <= b`, `x >= 0`; `None` if infeasible or unbounded.
///
/// Two-phase simplex with Bland's rule, comparing against `eps`. When the optimum is reached at
/// several points, `x` is the vertex the pivoting reaches; `val` is the optimum either way.
///
/// cost: \(O(\mathit{poly}(n, m))\) typical (exponential in the worst case); \(O(\mathit{rows} \cdot \mathit{cols})\) space
/// empty: with no variables, `Some((val: 0.0, x: []))` when every `b` entry is at least `-eps`
/// order: `x` indexed by column (variable)
/// traps: when `b.len != a.rows` or `c.len != a.cols`
/// example: `let r = simplex(Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1.0]]), [1.0, 2.0], [1.0, 1.0])!; abs(r.val - 3.0) < 1e-9 and abs(r.x[1] - 2.0) < 1e-9`
/// example: `simplex(Mat<F64>.from_rows([[1.0]]), [-1.0], [1.0]) == None`
/// example: `simplex(Mat<F64>.from_rows([[-1.0]]), [0.0], [1.0]) == None`
pub fn simplex(
    a: Mat<F64>,
    b: List<F64>,
    c: List<F64>,
    eps: F64 = 1e-9,
) -> ?(val: F64, x: List<F64>):
    assert b.len == a.rows, "simplex requires b.len == a.rows"
    assert c.len == a.cols, "simplex requires c.len == a.cols"
    let (m, n) = (a.rows, a.cols)
    if n == 0:
        for bi in b:
            if bi < -eps: return None
        return Some((val: 0.0, x: []))
    let w = n + 2
    var d = List.filled((m + 2) * w, 0.0)
    var nn = (0..n).collect()
    nn.push(-1)
    var bb = [n + i for i in 0..m]
    for i in 0..m:
        for j in 0..n: d[i * w + j] = a.get(i, j)
        d[i * w + n] = -1.0
        d[i * w + n + 1] = b[i]
    for j in 0..n: d[m * w + j] = -c[j]
    d[(m + 1) * w + n] = 1.0
    fn pivot(r: Int, s: Int):
        let inv = 1.0 / d[r * w + s]
        for i in 0..m + 2:
            if i != r and abs(d[i * w + s]) > eps:
                let f = d[i * w + s] * inv
                for j in 0..w: d[i * w + j] -= d[r * w + j] * f
                d[i * w + s] = d[r * w + s] * f
        for j in 0..w:
            if j != s: d[r * w + j] *= inv
        for i in 0..m + 2:
            if i != r: d[i * w + s] *= -inv
        d[r * w + s] = inv
        (bb[r], nn[s]) = (nn[s], bb[r])
    fn run(phase: Int) -> Bool:
        // Bland's rule breaks exact ties by index, as kactl's LPSolver does.
        let x = m + phase - 1
        loop:
            var pick: ?Int = None
            for j in 0..=n:
                if nn[j] != -phase:
                    if let t = pick:
                        if (d[x * w + j], nn[j]) < (d[x * w + t], nn[t]): pick = j
                    else: pick = j
            let s = pick!
            if d[x * w + s] >= -eps: return true
            var row: ?Int = None
            for i in 0..m:
                if d[i * w + s] > eps:
                    if let r = row:
                        let qi = d[i * w + n + 1] / d[i * w + s]
                        let qr = d[r * w + n + 1] / d[r * w + s]
                        if (qi, bb[i]) < (qr, bb[r]): row = i
                    else: row = i
            let r = row ?? return false
            pivot(r, s)
    if m > 0:
        var r = 0
        for i in 1..m:
            if d[i * w + n + 1] < d[r * w + n + 1]: r = i
        if d[r * w + n + 1] < -eps:
            pivot(r, n)
            if not run(2) or d[(m + 1) * w + n + 1] < -eps: return None
            for i in 0..m:
                if bb[i] == -1:
                    var s = 0
                    for j in 1..=n:
                        if (d[i * w + j], nn[j]) < (d[i * w + s], nn[s]): s = j
                    pivot(i, s)
    if not run(1): return None
    var x = List.filled(n, 0.0)
    for i in 0..m:
        if bb[i] >= 0 and bb[i] < n: x[bb[i]] = d[i * w + n + 1]
    Some((val: d[m * w + n + 1], x:))

@test
fn matrix_arithmetic_and_pow():
    type Z = Mod<1_000_000_007>
    let fib = Mat<Z>.from_rows([[Z(1), Z(1)], [Z(1), Z(0)]])
    let f10 = fib ** 10
    assert f10.get(0, 1) == Z(55) and f10.get(0, 0) == Z(89)
    let id = Mat<Z>.identity(2)
    assert fib * id == fib and id * fib == fib
    assert fib ** 0 == id
    assert (fib + id).get(0, 0) == Z(2) and (fib - fib) == Mat<Z>.zeros(2, 2)
    let rect = Mat<Int>.from_rows([[1, 2, 3], [4, 5, 6]])
    assert rect.transpose().rows == 3 and rect.transpose().cols == 2
    assert rect.mul_vec([1, 0, 2]) == [7, 16]

@test
fn exact_elimination_det_rank_solve_inverse():
    type Z = Mod<998_244_353>
    let a = Mat<Z>.from_rows([[Z(2), Z(1), Z(3)], [Z(1), Z(0), Z(2)], [Z(4), Z(1), Z(8)]])
    let inv = a.inverse()!
    assert a * inv == Mat<Z>.identity(3)
    assert a.det() * inv.det() == Z(1)
    assert a.rank() == 3
    let b = [Z(10), Z(5), Z(25)]
    let x = a.solve(b)!
    assert a.mul_vec(x) == b
    // Underdetermined system: verify particular + nullspace basis
    let u = Mat<Z>.from_rows([[Z(1), Z(2), Z(3)], [Z(2), Z(4), Z(6)]])
    assert u.rank() == 1
    let sp = u.solve_space([Z(5), Z(10)])!
    assert sp.1.len == 2
    assert u.mul_vec(sp.0) == [Z(5), Z(10)]
    for v in sp.1: assert u.mul_vec(v) == [Z(0), Z(0)]
    assert u.solve([Z(5), Z(11)]) == None

@test
fn real_elimination_matches_exact():
    let a = Mat<F64>.from_rows([[2.0, 1.0], [5.0, 3.0]])
    assert abs(a.det(eps: 1e-9) - 1.0) < 1e-9
    assert a.rank(eps: 1e-9) == 2
    let x = a.solve([4.0, 11.0], eps: 1e-9)!
    assert abs(x[0] - 1.0) < 1e-9 and abs(x[1] - 2.0) < 1e-9
    let inv = a.inverse(eps: 1e-9)!
    let prod = a * inv
    assert abs(prod.get(0, 0) - 1.0) < 1e-9 and abs(prod.get(0, 1)) < 1e-9
    assert abs(prod.get(1, 0)) < 1e-9 and abs(prod.get(1, 1) - 1.0) < 1e-9
    let sing = Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]])
    assert sing.rank(eps: 1e-9) == 1 and sing.inverse(eps: 1e-9) == None
    assert sing.solve([1.0, 3.0], eps: 1e-9) == None

@test
fn xor_basis_span_kth_and_merge():
    var xb = XorBasis.new()
    assert xb.rank == 0 and xb.contains(0) and not xb.contains(1)
    assert xb.add(3) and xb.add(5) and not xb.add(6)
    assert xb.rank == 2
    // Span of {3, 5} is {0, 3, 5, 6}
    assert xb.max_xor() == 6 and xb.max_xor(1) == 7 and xb.min_xor(7) == 1
    assert xb.kth(0) == 0 and xb.kth(1) == 3 and xb.kth(2) == 5 and xb.kth(3) == 6 and xb.kth(4) == None
    var other = XorBasis.new()
    assert other.add(8)
    xb.merge(other)
    assert xb.rank == 3 and xb.max_xor() == 14
    let m = XorBasis.monoid()
    var b1 = XorBasis.new()
    let _ = b1.add(5)
    var b2 = XorBasis.new()
    let _ = b2.add(10)
    let merged = m.op(b1, b2)
    assert merged.rank == 2 and merged.max_xor() == 15
