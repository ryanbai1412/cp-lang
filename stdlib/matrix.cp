//! Dense matrices, exact and floating-point Gaussian elimination, and GF(2) XOR bases.
//!
//! `Mat<T>` stores an `rows x cols` matrix in flat row-major order (`data[r * cols + c]`).
//! Exact elimination (`det`, `rank`, `solve`, `solve_space`, `inverse`) works over any field
//! type with `T.zero`, `T.one`, exact `==`, and `/` (such as `Mod<P>` for prime `P`).
//! Floating-point elimination uses partial pivoting with an explicit tolerance `eps` via
//! `det_real`, `rank_real`, `solve_real`, and `inverse_real`.
use std.algebra.{Monoid}
use std.mod.{Mod}

/// A dense `rows x cols` matrix stored flat in row-major order.
pub struct Mat<T>:
    rows: Int
    cols: Int
    data: List<T>

    /// A `rows x cols` matrix filled with `fill`. O(rows * cols).
    pub fn new(rows: Int, cols: Int, fill: T) -> Mat<T>:
        assert rows >= 0 and cols >= 0
        Mat(rows, cols, List.filled(rows * cols, fill))

    /// A `rows x cols` zero matrix. O(rows * cols).
    pub fn zeros(rows: Int, cols: Int) -> Mat<T>:
        assert rows >= 0 and cols >= 0
        Mat(rows, cols, List.filled(rows * cols, T.zero))

    /// The `n x n` identity matrix. O(n^2).
    pub fn identity(n: Int) -> Mat<T>:
        assert n >= 0
        var d = List.filled(n * n, T.zero)
        for i in 0..n: d[i * n + i] = T.one
        Mat(n, n, d)

    /// Build a `rows x cols` matrix from a flat row-major list of length `rows * cols`. O(rows * cols).
    pub fn from_flat(rows: Int, cols: Int, data: List<T>) -> Mat<T>:
        assert rows >= 0 and cols >= 0 and data.len == rows * cols
        Mat(rows, cols, data)

    /// Build a matrix from a list of equal-length rows. O(rows * cols).
    pub fn from_rows(grid: List<List<T>>) -> Mat<T>:
        let r = grid.len
        if r == 0: return Mat(0, 0, List<T>())
        let c = grid[0].len
        var d = List<T>()
        for row in grid:
            assert row.len == c, "all matrix rows must have equal length"
            for x in row: d.push(x)
        Mat(r, c, d)

    /// Entry at `(r, c)`. O(1).
    pub fn get(self, r: Int, c: Int) -> T = data[r * cols + c]

    /// Set entry at `(r, c)`. O(1).
    pub fn set(mut self, r: Int, c: Int, val: T):
        data[r * cols + c] = val

    /// Entry at `(r, c)`, lent in place: `m.at(i, j) += x`. O(1).
    pub fn at(self, r: Int, c: Int) lends mut T:
        lend mut data[r * cols + c]

    /// Transpose (`cols x rows`). O(rows * cols).
    pub fn transpose(self) -> Mat<T>:
        var d = List.filled(rows * cols, T.zero)
        for i in 0..rows:
            for j in 0..cols: d[j * rows + i] = data[i * cols + j]
        Mat(cols, rows, d)

    /// Matrix-vector product `self * v`. O(rows * cols).
    pub fn mul_vec(self, v: List<T>) -> List<T>:
        assert v.len == cols, "vector length must match matrix columns"
        var out = List.filled(rows, T.zero)
        for i in 0..rows:
            var acc = T.zero
            let off = i * cols
            for j in 0..cols: acc = acc + data[off + j] * v[j]
            out[i] = acc
        out

    /// Matrix multiplication monoid for `n x n` matrices (usable in `SegTree`).
    pub fn product(n: Int) -> Monoid<Mat<T>> = Monoid(Mat<T>.identity(n), (x, y) => x * y)

    /// Determinant over an exact field (such as `Mod<P>` for prime `P`). O(n^3).
    pub fn det(self) -> T:
        assert rows == cols, "det requires a square matrix"
        let n = rows
        var m = [x for x in data]
        var ans = T.one
        for col in 0..n:
            var piv = col
            while piv < n and m[piv * n + col] == T.zero: piv += 1
            if piv == n: return T.zero
            if piv != col:
                for c in col..n: m.swap(col * n + c, piv * n + c)
                ans = T.zero - ans
            let lead = m[col * n + col]
            ans = ans * lead
            let inv_l = T.one / lead
            for r in col + 1..n:
                let f = m[r * n + col]
                if f == T.zero: continue
                let scale = f * inv_l
                for c in col..n: m[r * n + c] = m[r * n + c] - scale * m[col * n + c]
        ans

    /// Rank over an exact field. O(rows * cols * min(rows, cols)).
    pub fn rank(self) -> Int:
        var m = [x for x in data]
        var r = 0
        for col in 0..cols:
            if r == rows: break
            var piv = r
            while piv < rows and m[piv * cols + col] == T.zero: piv += 1
            if piv == rows: continue
            if piv != r:
                for c in col..cols: m.swap(r * cols + c, piv * cols + c)
            let inv_l = T.one / m[r * cols + col]
            for i in r + 1..rows:
                let f = m[i * cols + col]
                if f == T.zero: continue
                let scale = f * inv_l
                for c in col..cols: m[i * cols + c] = m[i * cols + c] - scale * m[r * cols + c]
            r += 1
        r

    /// Solve `self * x = b` over an exact field. Returns `(particular, nullspace_basis)` or `none`.
    /// Every solution has the form `particular + sum(c_i * basis[i])`. O(rows * cols * min(rows, cols)).
    pub fn solve_space(self, b: List<T>) -> ?(List<T>, List<List<T>>):
        assert b.len == rows
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
            var piv = r
            while piv < rows and aug[piv * stride + col] == T.zero: piv += 1
            if piv == rows: continue
            if piv != r:
                for c in col..=k: aug.swap(r * stride + c, piv * stride + c)
            let inv_l = T.one / aug[r * stride + col]
            for c in col..=k: aug[r * stride + c] = aug[r * stride + c] * inv_l
            for i in 0..rows:
                if i == r: continue
                let f = aug[i * stride + col]
                if f == T.zero: continue
                for c in col..=k: aug[i * stride + c] = aug[i * stride + c] - f * aug[r * stride + c]
            pivot_row[col] = r
            r += 1
        for i in r..rows:
            if aug[i * stride + k] != T.zero: return none
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

    /// Solve `self * x = b` over an exact field. Returns one solution (free variables 0) or `none`.
    pub fn solve(self, b: List<T>) -> ?List<T>:
        let res = self.solve_space(b)
        if res == none: return none
        res!.0

    /// Multiplicative inverse of a square matrix over an exact field, or `none` if singular. O(n^3).
    pub fn inverse(self) -> ?Mat<T>:
        assert rows == cols, "inverse requires a square matrix"
        let n = rows
        let stride = 2 * n
        var aug = List.filled(n * stride, T.zero)
        for i in 0..n:
            for j in 0..n: aug[i * stride + j] = data[i * n + j]
            aug[i * stride + n + i] = T.one
        for col in 0..n:
            var piv = col
            while piv < n and aug[piv * stride + col] == T.zero: piv += 1
            if piv == n: return none
            if piv != col:
                for c in col..stride: aug.swap(col * stride + c, piv * stride + c)
            let inv_l = T.one / aug[col * stride + col]
            for c in col..stride: aug[col * stride + c] = aug[col * stride + c] * inv_l
            for i in 0..n:
                if i == col: continue
                let f = aug[i * stride + col]
                if f == T.zero: continue
                for c in col..stride: aug[i * stride + c] = aug[i * stride + c] - f * aug[col * stride + c]
        var out = List.filled(n * n, T.zero)
        for i in 0..n:
            for j in 0..n: out[i * n + j] = aug[i * stride + n + j]
        Mat(n, n, out)

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
        var out = List.filled(r * c, T.zero)
        unchecked:
            for i in 0..r:
                let ai = i * m
                let oi = i * c
                for k in 0..m:
                    let f = a.data[ai + k]
                    let bk = k * c
                    for j in 0..c: out[oi + j] = out[oi + j] + f * b.data[bk + j]
        Mat(r, c, out)

impl Pow for Mat<T>:
    fn pow(a, e: Int) -> Mat<T>:
        assert a.rows == a.cols and e >= 0
        var base = a
        var acc = Mat<T>.identity(a.rows)
        var k = e
        while k > 0:
            if (k & 1) == 1: acc = acc * base
            k >>= 1
            if k > 0: base = base * base
        acc

/// Determinant of a floating-point square matrix via partial pivoting with tolerance `eps`. O(n^3).
pub fn det_real(a: Mat<F64>, eps: F64 = 1e-9) -> F64:
    assert a.rows == a.cols, "det_real requires a square matrix"
    let n = a.rows
    var m = [x for x in a.data]
    var ans = 1.0
    for col in 0..n:
        var piv = col
        for r in col + 1..n:
            if abs(m[r * n + col]) > abs(m[piv * n + col]): piv = r
        if abs(m[piv * n + col]) <= eps: return 0.0
        if piv != col:
            for c in col..n: m.swap(col * n + c, piv * n + c)
            ans = -ans
        let lead = m[col * n + col]
        ans *= lead
        for r in col + 1..n:
            let scale = m[r * n + col] / lead
            for c in col..n: m[r * n + c] -= scale * m[col * n + c]
    ans

/// Rank of a floating-point matrix via partial pivoting with tolerance `eps`. O(r * c * min(r, c)).
pub fn rank_real(a: Mat<F64>, eps: F64 = 1e-9) -> Int:
    let (rows, cols) = (a.rows, a.cols)
    var m = [x for x in a.data]
    var r = 0
    for col in 0..cols:
        if r == rows: break
        var piv = r
        for i in r + 1..rows:
            if abs(m[i * cols + col]) > abs(m[piv * cols + col]): piv = i
        if abs(m[piv * cols + col]) <= eps: continue
        if piv != r:
            for c in col..cols: m.swap(r * cols + c, piv * cols + c)
        let lead = m[r * cols + col]
        for i in r + 1..rows:
            let scale = m[i * cols + col] / lead
            for c in col..cols: m[i * cols + c] -= scale * m[r * cols + c]
        r += 1
    r

/// Solve `a * x = b` over `F64` via partial pivoting with tolerance `eps`. Returns `none` if inconsistent.
pub fn solve_real(a: Mat<F64>, b: List<F64>, eps: F64 = 1e-9) -> ?List<F64>:
    assert b.len == a.rows
    let (rows, k) = (a.rows, a.cols)
    let stride = k + 1
    var aug = List.filled(rows * stride, 0.0)
    for i in 0..rows:
        for j in 0..k: aug[i * stride + j] = a.data[i * k + j]
        aug[i * stride + k] = b[i]
    var pivot_row = List.filled(k, -1)
    var r = 0
    for col in 0..k:
        if r == rows: break
        var piv = r
        for i in r + 1..rows:
            if abs(aug[i * stride + col]) > abs(aug[piv * stride + col]): piv = i
        if abs(aug[piv * stride + col]) <= eps: continue
        if piv != r:
            for c in col..=k: aug.swap(r * stride + c, piv * stride + c)
        let lead = aug[r * stride + col]
        for c in col..=k: aug[r * stride + c] /= lead
        for i in 0..rows:
            if i == r: continue
            let f = aug[i * stride + col]
            if abs(f) <= eps: continue
            for c in col..=k: aug[i * stride + c] -= f * aug[r * stride + c]
        pivot_row[col] = r
        r += 1
    for i in r..rows:
        if abs(aug[i * stride + k]) > eps: return none
    var x = List.filled(k, 0.0)
    for col in 0..k:
        if pivot_row[col] != -1: x[col] = aug[pivot_row[col] * stride + k]
    x

/// Inverse of a floating-point square matrix via partial pivoting with tolerance `eps`, or `none` if singular.
pub fn inverse_real(a: Mat<F64>, eps: F64 = 1e-9) -> ?Mat<F64>:
    assert a.rows == a.cols, "inverse_real requires a square matrix"
    let n = a.rows
    let stride = 2 * n
    var aug = List.filled(n * stride, 0.0)
    for i in 0..n:
        for j in 0..n: aug[i * stride + j] = a.data[i * n + j]
        aug[i * stride + n + i] = 1.0
    for col in 0..n:
        var piv = col
        for i in col + 1..n:
            if abs(aug[i * stride + col]) > abs(aug[piv * stride + col]): piv = i
        if abs(aug[piv * stride + col]) <= eps: return none
        if piv != col:
            for c in col..stride: aug.swap(col * stride + c, piv * stride + c)
        let lead = aug[col * stride + col]
        for c in col..stride: aug[col * stride + c] /= lead
        for i in 0..n:
            if i == col: continue
            let f = aug[i * stride + col]
            if abs(f) <= eps: continue
            for c in col..stride: aug[i * stride + c] -= f * aug[col * stride + c]
    var out = Mat<F64>.zeros(n, n)
    for i in 0..n:
        for j in 0..n: out.set(i, j, aug[i * stride + n + j])
    out

/// Linear basis over GF(2) for non-negative 63-bit integers (`0 <= x < 2^63`).
/// Every operation is O(W) where W <= 63 (`merge` is O(W * rank)).
pub struct XorBasis:
    basis: List<Int>
    size: Int

    /// Empty basis of rank 0. O(W).
    pub fn new() -> XorBasis = XorBasis(List.filled(63, 0), 0)

    /// Number of linearly independent vectors in the basis. O(1).
    pub @get fn rank(self) -> Int = size

    /// Insert `x >= 0` into the basis. Returns `true` iff the rank increased. O(W).
    pub fn add(mut self, x: Int) -> Bool:
        assert x >= 0
        var v = x
        var b = 62
        while b >= 0 and v > 0:
            if ((v >> b) & 1) != 0:
                if basis[b] == 0:
                    basis[b] = v
                    size += 1
                    return true
                v ^= basis[b]
            b -= 1
        false

    /// Reduce `x >= 0` against the basis (0 iff `x` is in the span). O(W).
    pub fn reduce(self, x: Int) -> Int:
        assert x >= 0
        var v = x
        var b = 62
        while b >= 0 and v > 0:
            if ((v >> b) & 1) != 0 and basis[b] != 0: v ^= basis[b]
            b -= 1
        v

    /// Whether `x >= 0` is a XOR-combination of basis vectors. O(W).
    pub fn contains(self, x: Int) -> Bool = self.reduce(x) == 0

    /// Maximum value of `init ^ s` over all `s` in the span. O(W).
    pub fn max_xor(self, init: Int = 0) -> Int:
        assert init >= 0
        var v = init
        var b = 62
        while b >= 0:
            if basis[b] != 0 and (v ^ basis[b]) > v: v ^= basis[b]
            b -= 1
        v

    /// Minimum value of `init ^ s` over all `s` in the span. O(W).
    pub fn min_xor(self, init: Int = 0) -> Int:
        assert init >= 0
        var v = init
        var b = 62
        while b >= 0:
            if basis[b] != 0 and (v ^ basis[b]) < v: v ^= basis[b]
            b -= 1
        v

    /// The 0-indexed `k`-th smallest value in the span, or `none` if `k < 0` or `k >= 2^rank`. O(W^2).
    pub fn kth(self, k: Int) -> ?Int:
        if k < 0: return none
        if size < 63 and k >= (1 << size): return none
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
            if ((k >> i) & 1) != 0: ans ^= rref[i]
        ans

    /// Merge another basis into `self`. O(W * o.rank).
    pub fn merge(mut self, o: XorBasis):
        for b in 0..63:
            if o.basis[b] != 0:
                let _ = self.add(o.basis[b])

    /// Monoid under basis merge (usable in `SegTree`).
    pub fn monoid() -> Monoid<XorBasis> = Monoid(XorBasis.new(), merge_xor)

fn merge_xor(a: XorBasis, b: XorBasis) -> XorBasis:
    var out = XorBasis.new()
    out.merge(a)
    out.merge(b)
    out

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
    assert u.solve([Z(5), Z(11)]) == none

@test
fn real_elimination_matches_exact():
    let a = Mat<F64>.from_rows([[2.0, 1.0], [5.0, 3.0]])
    assert abs(det_real(a) - 1.0) < 1e-9
    assert rank_real(a) == 2
    let x = solve_real(a, [4.0, 11.0])!
    assert abs(x[0] - 1.0) < 1e-9 and abs(x[1] - 2.0) < 1e-9
    let inv = inverse_real(a)!
    let prod = a * inv
    assert abs(prod.get(0, 0) - 1.0) < 1e-9 and abs(prod.get(0, 1)) < 1e-9
    assert abs(prod.get(1, 0)) < 1e-9 and abs(prod.get(1, 1) - 1.0) < 1e-9
    let sing = Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]])
    assert rank_real(sing) == 1 and inverse_real(sing) == none and solve_real(
        sing,
        [1.0, 3.0],
    ) == none

@test
fn xor_basis_span_kth_and_merge():
    var xb = XorBasis.new()
    assert xb.rank == 0 and xb.contains(0) and not xb.contains(1)
    assert xb.add(3) and xb.add(5) and not xb.add(6)
    assert xb.rank == 2
    // Span of {3, 5} is {0, 3, 5, 6}
    assert xb.max_xor() == 6 and xb.max_xor(1) == 7 and xb.min_xor(7) == 1
    assert xb.kth(0) == 0 and xb.kth(1) == 3 and xb.kth(2) == 5 and xb.kth(3) == 6 and xb.kth(4) == none
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
