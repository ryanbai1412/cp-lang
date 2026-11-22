//! Polynomials found from the values of a brute force: is the answer a polynomial in `n`,
//! of what degree, with which coefficients, and what is it at `n = 10^18`?
//! `std.spec` has the integer finite-difference tools (`poly_degree`, `poly_at`); this
//! module adds the same test modulo a prime, Lagrange evaluation in O(k), and the exact
//! coefficients as fractions. For coefficients modulo a prime through arbitrary points,
//! see `lib.math.poly.interpolate`.
use std.mod.{Mod}
use std.spec.{poly_degree}
use lib.math.frac.{Frac}

/// The degree of the polynomial through `ys` (taken at 0, 1, 2, ...) modulo the prime
/// `P`, by repeated differences, if at least one extra value confirms it. `none` when
/// `ys` is not visibly a polynomial; the zero sequence has degree -1. Needs
/// `ys.len <= P`. O(n^2).
pub fn poly_degree_mod<P>(ys: List<Mod<P>>) -> ?Int:
    var row = [y for y in ys]
    var (degree, depth) = (-1, 0)
    while not row.is_empty:
        if row[0] != 0: degree = depth
        row = [row[i + 1] - row[i] for i in 0..row.len - 1]
        depth += 1
    if degree + 1 < ys.len: degree else: none

/// The value at any `n` of the polynomial of degree below `ys.len` through `ys` (taken
/// at 0, 1, 2, ...), modulo the prime `P` (Lagrange's formula on consecutive points).
/// O(k) after one modular inverse, for `k = ys.len <= P`.
pub fn lagrange_at<P>(ys: List<Mod<P>>, n: Int) -> Mod<P>:
    let k = ys.len
    let one: Mod<P> = 1
    if 0 <= n and n < k: return ys[n]
    // before[i] = (n - 0) * ... * (n - (i - 1)), after[i] = (n - (i + 1)) * ... * (n - (k - 1))
    var before = List.filled(k + 1, one)
    var after = List.filled(k + 1, one)
    for i in 0..k: before[i + 1] = before[i] * Mod<P>(n - i)
    for i in (0..k).rev: after[i] = after[i + 1] * Mod<P>(n - i)
    var inv_fact = List.filled(k, one)
    var fact = one
    for i in 1..k: fact *= Mod<P>(i)
    inv_fact[k - 1] = fact.inv()
    for i in (1..k).rev: inv_fact[i - 1] = inv_fact[i] * Mod<P>(i)
    var total: Mod<P> = 0
    for i in 0..k:
        let term = ys[i] * before[i] * after[i + 1] * inv_fact[i] * inv_fact[k - 1 - i]
        if (k - 1 - i) % 2 == 0: total += term else: total -= term
    total

/// The coefficients, lowest degree first, of the polynomial of least degree through
/// every `(x, y)` in `pts` (distinct `x`), as exact fractions; trailing zeros are cut,
/// so the zero polynomial is `[]`. Newton's divided differences, O(n^2).
pub fn exact_poly(pts: List<(Int, Int)>) -> List<Frac>:
    let n = pts.len
    let zero = Frac.new(0)
    // dd[i] becomes the divided difference over pts[0..=i]
    var dd = [Frac.new(y) for (_, y) in pts]
    for depth in 1..n:
        for i in (depth..n).rev:
            dd[i] = (dd[i] - dd[i - 1]) / Frac.new(pts[i].0 - pts[i - depth].0)
    // Horner over the Newton basis: coef = dd[i] + (x - x_i) * coef
    var coef = List<Frac>()
    for i in (0..n).rev:
        let shift = Frac.new(pts[i].0)
        var next = List.filled(coef.len + 1, zero)
        for j in 0..coef.len:
            next[j + 1] += coef[j]
            next[j] -= coef[j] * shift
        next[0] += dd[i]
        coef = next
    while not coef.is_empty and coef.last == zero: coef.pop()
    coef

/// The polynomial `coef` (lowest degree first) at `x`, exactly.
pub fn exact_poly_at(coef: List<Frac>, x: Frac) -> Frac:
    var acc = Frac.new(0)
    for i in (0..coef.len).rev: acc = acc * x + coef[i]
    acc

/// The closed form of a sequence taken at 0, 1, 2, ...: the coefficients of the
/// polynomial through `xs`, when the data confirms one (`std.spec.poly_degree`), else
/// `none`.
pub fn closed_form(xs: List<Int>) -> ?List<Frac>:
    if let d = poly_degree(xs): exact_poly([(i, xs[i]) for i in 0..=d]) else: none

/// A polynomial written out, highest degree first: `1/3 n^3 + 1/2 n^2 + 1/6 n`.
pub fn show_poly(coef: List<Frac>) -> Str:
    var out = ""
    for i in (0..coef.len).rev:
        let c = coef[i]
        if c.num == 0: continue
        if out.len > 0: out += (if c.num < 0: " - " else: " + ") elif c.num < 0: out += "-"
        let mag = if c.den == 1: "{abs(c.num)}" else: "{abs(c.num)}/{c.den}"
        let power = if i == 0: "" elif i == 1: "n" else: "n^{i}"
        if i == 0: out += mag
        elif mag == "1": out += power
        else: out += mag + " " + power
    if out.len == 0: "0" else: out

@test
fn degrees_modulo_a_prime():
    type M = Mod<998244353>
    let cubic = [M(x * x * x - 4 * x + 7) for x in 0..9]
    assert poly_degree_mod(cubic) == 3
    assert poly_degree_mod([M(5), M(5), M(5)]) == 0 and poly_degree_mod([M(0), M(0)]) == -1
    assert poly_degree_mod([M(2) ** i for i in 0..12]) == none
    // three points always fit a parabola, so nothing confirms it
    assert poly_degree_mod([M(3), M(1), M(4)]) == none
    // halves are fine modulo a prime: n * (n + 1) / 2 has degree 2
    assert poly_degree_mod([M(n) * M(n + 1) / M(2) for n in 0..6]) == 2

@test
fn lagrange_matches_horner_everywhere():
    type M = Mod<1000000007>
    for _ in 0..30:
        let deg = rand(0, 7)
        let coef = [rand(-9, 9) for _ in 0..=deg]
        let ys = [M(sum(coef[i] * x ** i for i in 0..=deg)) for x in 0..=deg]
        for n in [0, deg, deg + 1, 50, -3, 123456789012]:
            var want: M = 0
            for i in (0..=deg).rev: want = want * M(n) + M(coef[i])
            assert lagrange_at(ys, n) == want
    assert lagrange_at([M(42)], 10 ** 18) == 42

@test
fn sums_of_powers_far_out():
    // 1^k + ... + n^k is a polynomial of degree k + 1: k + 2 brute-force values fix it
    type M = Mod<1000000007>
    for k in 0..=6:
        var ys = [M(0)]
        for x in 1..=k + 2: ys.push(ys.last + M(x) ** k)
        assert poly_degree_mod(ys) == k + 1
        let want: M = sum(M(x) ** k for x in 1..=300)
        assert lagrange_at(ys, 300) == want
    // n = 10^18, k = 1: n * (n + 1) / 2
    let n = 10 ** 18
    assert lagrange_at([M(0), M(1), M(3)], n) == M(n) * M(n + 1) / M(2)

@test
fn exact_coefficients_are_fractions():
    // 0, 1, 5, 14, 30: sums of squares, n (n + 1) (2n + 1) / 6
    let squares = exact_poly([(n, sum(x * x for x in 0..=n)) for n in 0..5])
    assert squares == [Frac(0, 1), Frac(1, 6), Frac(1, 2), Frac(1, 3)]
    assert show_poly(squares) == "1/3 n^3 + 1/2 n^2 + 1/6 n"
    assert exact_poly_at(squares, Frac.new(100)) == Frac.new(338350)
    assert exact_poly_at(squares, Frac.new(1, 2)) == Frac(1, 4)
    assert exact_poly(List<(Int, Int)>()) == [] and exact_poly([(3, 0), (9, 0)]) == []
    assert exact_poly([(-2, 7)]) == [Frac(7, 1)]
    // points in any order, at any distinct x
    for _ in 0..30:
        let deg = rand(0, 5)
        let coef = [rand(-5, 5) for _ in 0..=deg]
        var xs = [x for x in -6..=6]
        shuffle(mut xs)
        let pts = [(xs[i], sum(coef[j] * xs[i] ** j for j in 0..=deg)) for i in 0..=deg + 2]
        var want = [Frac.new(c) for c in coef]
        while not want.is_empty and want.last == Frac.new(0): want.pop()
        assert exact_poly(pts) == want

@test
fn closed_forms_need_confirmation():
    let tri = [n * (n + 1) / 2 for n in 0..7]
    assert closed_form(tri) == [Frac(0, 1), Frac(1, 2), Frac(1, 2)]
    assert show_poly(closed_form(tri)!) == "1/2 n^2 + 1/2 n"
    assert closed_form([1, 2, 4, 8, 16, 32]) == none and closed_form([3, 1, 4]) == none
    assert closed_form([0, 0, 0]) == [] and show_poly([]) == "0"
    assert show_poly([Frac(-7, 1), Frac(0, 1), Frac(-1, 1)]) == "-n^2 - 7"
    assert show_poly([Frac(2, 1), Frac(-3, 2)]) == "-3/2 n + 2" and show_poly([Frac(1, 1)]) == "1"
