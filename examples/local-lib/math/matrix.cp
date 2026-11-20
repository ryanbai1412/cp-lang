//! Square matrices over `Mod<P>`, for linear recurrences and counting walks.

/// An `n` by `n` matrix. Multiplication is O(n^3), `pow` O(n^3 log e).
use std.mod.{Mod}
use std.algebra.{Monoid}
pub struct Mat<P>(n: Int, a: List<List<Mod<P>>>):
    pub fn zero(n: Int) -> Mat<P> = Mat(n, [[Mod<P>(0) for _ in 0..n] for _ in 0..n])

    pub fn identity(n: Int) -> Mat<P> = Mat(
        n,
        [[Mod<P>(if i == j: 1 else: 0) for j in 0..n] for i in 0..n],
    )

    /// From rows of plain integers.
    pub fn of(rows: List<List<Int>>) -> Mat<P> = Mat(
        rows.len,
        [[Mod<P>(x) for x in row] for row in rows],
    )

    /// The entry in row `i`, column `j`, lent in place: `m.at(i, j) += x` updates the matrix.
    pub fn at(self, i: Int, j: Int) lends mut Mod<P>:
        lend mut a[i][j]

    pub fn pow(self, e: Int) -> Mat<P>:
        var (base, acc, k) = (self, Mat<P>.identity(n), e)
        while k > 0:
            if k % 2 == 1: acc *= base
            base *= base
            k /= 2
        acc

    /// Matrix multiplication as a monoid, so a `SegTree` can hold matrices.
    pub fn product(n: Int) -> Monoid<Mat<P>> = Monoid(Mat<P>.identity(n), (x, y) => x * y)

    /// The matrix times a column vector.
    pub fn apply(self, v: List<Mod<P>>) -> List<Mod<P>>:
        var out = List.filled(n, Mod<P>(0))
        for i in 0..n:
            for j in 0..n: out[i] += a[i][j] * v[j]
        out

    /// Determinant modulo prime `P` by Gaussian elimination. O(n^3).
    pub fn det(self) -> Mod<P>:
        var m = [[a[i][j] for j in 0..n] for i in 0..n]
        var ans: Mod<P> = 1
        for col in 0..n:
            var piv = col
            while piv < n and m[piv][col] == 0: piv += 1
            if piv == n: return 0
            if piv != col:
                let tmp = m[col]
                m[col] = m[piv]
                m[piv] = tmp
                ans = -ans
            let lead = m[col][col]
            ans *= lead
            let inv_l = lead.inv
            for r in col + 1..n:
                if m[r][col] == 0: continue
                let f = m[r][col] * inv_l
                for c in col..n: m[r][c] -= f * m[col][c]
        ans

    /// Rank of the matrix modulo prime `P`. O(n^3).
    pub fn rank(self) -> Int:
        var m = [[a[i][j] for j in 0..n] for i in 0..n]
        var r = 0
        for col in 0..n:
            var piv = r
            while piv < n and m[piv][col] == 0: piv += 1
            if piv == n: continue
            if piv != r:
                let tmp = m[r]
                m[r] = m[piv]
                m[piv] = tmp
            let inv_l = m[r][col].inv
            for i in r + 1..n:
                if m[i][col] == 0: continue
                let f = m[i][col] * inv_l
                for c in col..n: m[i][c] -= f * m[r][c]
            r += 1
        r

    /// Solve `A * x = b` modulo prime `P`. Returns one solution `x`, or `none` if inconsistent. O(n^3).
    pub fn solve(self, b: List<Mod<P>>) -> ?List<Mod<P>> = solve_linear(a, b)

/// Solve a rectangular linear system `A * x = b` (`m` equations in `k` unknowns) modulo prime `P`.
/// Returns one solution `x` (free variables set to 0), or `none` if inconsistent. O(m * k * min(m, k)).
pub fn solve_linear<P>(rows: List<List<Mod<P>>>, b: List<Mod<P>>) -> ?List<Mod<P>>:
    let m = rows.len
    if m == 0: return []
    let k = rows[0].len
    var aug = [[rows[i][j] for j in 0..k] + [b[i]] for i in 0..m]
    var pivot_col = List.filled(k, -1)
    var r = 0
    for col in 0..k:
        if r == m: break
        var piv = r
        while piv < m and aug[piv][col] == 0: piv += 1
        if piv == m: continue
        if piv != r:
            aug.swap(r, piv)
        let inv_l = aug[r][col].inv
        for c in col..=k: aug[r][c] *= inv_l
        for i in 0..m:
            if i == r or aug[i][col] == 0: continue
            let f = aug[i][col]
            for c in col..=k: aug[i][c] -= f * aug[r][c]
        pivot_col[col] = r
        r += 1
    for i in r..m:
        if aug[i][k] != 0: return none
    var x = List.filled(k, Mod<P>(0))
    for col in 0..k:
        if pivot_col[col] != -1: x[col] = aug[pivot_col[col]][k]
    x

impl Mul for Mat<P>:
    fn mul(x, y):
        var out = Mat<P>.zero(x.n)
        for i in 0..x.n:
            for k in 0..x.n:
                let f = x.a[i][k]
                for j in 0..x.n: out.at(i, j) += f * y.a[k][j]
        out

impl Add for Mat<P>:
    fn add(x, y) = Mat(x.n, [[x.a[i][j] + y.a[i][j] for j in 0..x.n] for i in 0..x.n])

@test
fn fibonacci_by_matrix_power():
    type M = Mod<1000000007>
    let step = Mat<1000000007>.of([[1, 1], [1, 0]])
    var (a, b) = (M(0), M(1))
    for n in 0..60:
        assert step.pow(n).apply([M(1), M(0)])[1] == a
        (a, b) = (b, a + b)
    assert step.pow(0) == Mat<1000000007>.identity(2)

@test
fn counting_walks_matches_dp():
    type M = Mod<998244353>
    let n = 5
    let adj = [[rand(0, 1) for _ in 0..n] for _ in 0..n]
    let g = Mat<998244353>.of(adj)
    var ways = [[M(Int(i == j)) for j in 0..n] for i in 0..n]
    for len in 0..12:
        assert g.pow(len).a == ways
        ways = [[sum_mod([ways[i][k] * M(adj[k][j]) for k in 0..n]) for j in 0..n] for i in 0..n]

fn sum_mod<P>(xs: List<Mod<P>>) -> Mod<P>:
    var t: Mod<P> = 0
    for x in xs: t += x
    t

@test
fn matrix_product_is_a_lawful_monoid():
    let ms = [
        Mat<7>.of([[1, 2], [3, 4]]),
        Mat<7>.of([[0, 1], [1, 0]]),
        Mat<7>.identity(2),
        Mat<7>.of([[5, 0], [6, 6]]),
    ]
    assert laws_hold(Mat<7>.product(2), ms)
    assert ms[0] + ms[1] == Mat<7>.of([[1, 3], [4, 4]])

@test
fn entries_are_lent_in_place():
    var m = Mat<7>.zero(2)
    m.at(0, 1) = 3
    m.at(0, 1) += 5
    m.at(1, 0) = m.at(0, 1) * Mod<7>(2)
    assert m == Mat<7>.of([[0, 1], [2, 0]])

@test
fn det_rank_and_solve_over_mod_prime():
    type M = Mod<1000000007>
    // Multiplicativity of det and exact solve on random invertible matrices
    for _ in 0..15:
        let n = rand(1, 5)
        let a = Mat<1000000007>.of([[rand(0, 20) for _ in 0..n] for _ in 0..n])
        let b = Mat<1000000007>.of([[rand(0, 20) for _ in 0..n] for _ in 0..n])
        assert (a * b).det() == a.det() * b.det()
        let x_true = [M(rand(0, 50)) for _ in 0..n]
        let rhs = a.apply(x_true)
        let sol = a.solve(rhs)
        assert sol != none and a.apply(sol!) == rhs
        if a.det().v != 0:
            assert a.rank() == n and sol! == x_true
    // Singular & inconsistent systems
    let sing = Mat<7>.of([[1, 2, 3], [2, 4, 6], [1, 0, 1]])
    assert sing.det() == 0
    assert sing.rank() == 2
    assert sing.solve([Mod<7>(1), Mod<7>(3), Mod<7>(1)]) == none
    let ok_sol = sing.solve([Mod<7>(1), Mod<7>(2), Mod<7>(3)])
    assert ok_sol != none and sing.apply(ok_sol!) == [Mod<7>(1), Mod<7>(2), Mod<7>(3)]
