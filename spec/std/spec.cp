//! Checking observations on small cases: is a table monotone, convex, unimodal or Monge, does a
//! cost satisfy the quadrangle inequality or a split table Knuth's bound, and which recurrence
//! does a sequence follow. Each checker returns the first witness that breaks the law (`None`
//! when it holds), so a brute force over small inputs can print it; the recurrence tools then
//! extend a pinned sequence to any index.

use std.mod.{Mod}

/// The first position whose element is below the one before it: `Some(i)` for the smallest
/// `i >= 1` with `xs[i] < xs[i - 1]`, or `None` when `xs` never decreases.
///
/// cost: O(n)
/// empty: `None`
/// traps: never
/// example: `decrease([1, 2, 2, 5]) == None`
/// example: `decrease([1, 3, 2, 1]) == Some(2)`
pub fn decrease<T>(xs: List<T>) -> ?Int:
    for i in 1..xs.len:
        if xs[i] < xs[i - 1]: return Some(i)
    None

/// The first position whose element is above the one before it: `Some(i)` for the smallest
/// `i >= 1` with `xs[i] > xs[i - 1]`, or `None` when `xs` never increases.
///
/// cost: O(n)
/// empty: `None`
/// traps: never
/// example: `increase([5, 2, 2, 1]) == None`
/// example: `increase([3, 1, 2, 0]) == Some(2)`
pub fn increase<T>(xs: List<T>) -> ?Int:
    for i in 1..xs.len:
        if xs[i] > xs[i - 1]: return Some(i)
    None

/// The first position where `xs` is not convex: `Some(i)` for the smallest `i` with
/// `xs[i - 1] + xs[i + 1] < 2 * xs[i]`, or `None` when every second difference is at least 0.
///
/// cost: O(n)
/// empty: `None` (fewer than 3 elements cannot break the law)
/// traps: never (overflow wraps in release, traps in debug)
/// example: `convexity_break([4, 1, 0, 1, 4]) == None`
/// example: `convexity_break([0, 1, 3, 4]) == Some(2)`
pub fn convexity_break(xs: List<Int>) -> ?Int:
    for i in 1..xs.len - 1:
        if xs[i - 1] + xs[i + 1] < 2 * xs[i]: return Some(i)
    None

/// The first position where `xs` is not concave: `Some(i)` for the smallest `i` with
/// `xs[i - 1] + xs[i + 1] > 2 * xs[i]`, or `None` when every second difference is at most 0.
///
/// cost: O(n)
/// empty: `None` (fewer than 3 elements cannot break the law)
/// traps: never (overflow wraps in release, traps in debug)
/// example: `concavity_break([0, 3, 4, 3]) == None`
/// example: `concavity_break([0, 3, 4, 7]) == Some(2)`
pub fn concavity_break(xs: List<Int>) -> ?Int:
    for i in 1..xs.len - 1:
        if xs[i - 1] + xs[i + 1] > 2 * xs[i]: return Some(i)
    None

/// The first position where `xs` stops having the shape `ternary_max` needs: strictly rising,
/// then a plateau only at the maximum, then strictly falling. `Some(i)` names the element that
/// breaks it, `None` when the shape holds. For `ternary_min`, check the negated values.
///
/// cost: O(n)
/// empty: `None`
/// traps: never
/// example: `unimodality_break([1, 3, 5, 5, 2]) == None`
/// example: `unimodality_break([1, 2, 2, 3, 1]) == Some(3)`
/// example: `unimodality_break([5, 3, 4]) == Some(2)`
pub fn unimodality_break(xs: List<Int>) -> ?Int:
    var phase = 0
    for i in 1..xs.len:
        let (a, b) = (xs[i - 1], xs[i])
        if phase == 0:
            if b == a: phase = 1
            elif b < a: phase = 2
        elif phase == 1:
            if b > a: return Some(i)
            if b < a: phase = 2
        elif b >= a:
            return Some(i)
    None

/// The first 2x2 block of `cost` over `0..rows` by `0..cols` that breaks the Monge law
/// `cost(i, j) + cost(i + 1, j + 1) <= cost(i, j + 1) + cost(i + 1, j)`, as its top-left corner,
/// rows first; `None` when the table is Monge. Adjacent blocks suffice: the law for any
/// `a < b`, `c < d` is a sum of them.
///
/// cost: O(rows * cols) calls of `cost`
/// empty: `None` when `rows < 2` or `cols < 2`
/// traps: never
/// example: `monge_break((i, j) => (i - j) * (i - j), 4, 4) == None`
/// example: `monge_break((i, j) => i * j, 3, 3) == Some((0, 0))`
pub fn monge_break(cost: (Int, Int) -> Int, rows: Int, cols: Int) -> ?(Int, Int):
    for i in 0..rows - 1:
        for j in 0..cols - 1:
            if cost(i, j) + cost(
                i + 1,
                j + 1,
            ) > cost(i, j + 1) + cost(i + 1, j): return Some((i, j))
    None

/// The leftmost column of a minimum in each row of `cost` over `0..rows` by `0..cols`, by brute
/// force: the reference that SMAWK, divide and conquer and Knuth's speed-up must reproduce.
///
/// cost: O(rows * cols) calls of `cost`
/// empty: `[]` when `rows == 0`
/// ties: the leftmost minimal column
/// order: one column per row, in row order
/// traps: when `rows > 0` and `cols < 1`
/// example: `argmins((i, j) => (i - j) * (i - j), 3, 4) == [0, 1, 2]`
/// example: `argmins((i, j) => 0, 2, 3) == [0, 0]`
pub fn argmins(cost: (Int, Int) -> Int, rows: Int, cols: Int) -> List<Int>:
    assert rows <= 0 or cols >= 1, "argmins requires cols >= 1"
    var out: List<Int> = []
    for i in 0..rows:
        var best = 0
        for j in 1..cols:
            if cost(i, j) < cost(i, best): best = j
        out.push(best)
    out

/// The first adjacent block of `cost(j, i)` over `0 <= j < i < n` that breaks the quadrangle
/// inequality `cost(a, c) + cost(a + 1, c + 1) <= cost(a, c + 1) + cost(a + 1, c)`, as `(a, c)`
/// with `a + 1 < c`, by `a` then `c`; `None` when it holds (the law `dnc_layer` relies on).
/// Adjacent blocks suffice: the law for any `a < b < c < d` is a sum of them. `cost` is asked
/// only for `j < i`.
///
/// cost: O(n^2) calls of `cost`
/// empty: `None` when `n < 4`
/// traps: never
/// example: `dnc_break((j, i) => (i - j) * (i - j), 6) == None`
/// example: `dnc_break((j, i) => 0 - (i - j) * (i - j), 4) == Some((0, 2))`
pub fn dnc_break(cost: (Int, Int) -> Int, n: Int) -> ?(Int, Int):
    for a in 0..n - 3:
        for c in a + 2..n - 1:
            if cost(a, c) + cost(
                a + 1,
                c + 1,
            ) > cost(a, c + 1) + cost(a + 1, c): return Some((a, c))
    None

/// The first interval `[i, j)` of cuts `0..=n` whose best split `opt(i, j)` breaks Knuth's bound
/// `opt(i, j - 1) <= opt(i, j) <= opt(i + 1, j)`, shortest intervals first, then by `i`; `None`
/// when the bound holds (the law `knuth` relies on). `opt` is asked only for `j - i >= 2`.
///
/// cost: O(n^2) calls of `opt`
/// empty: `None` when `n < 3`
/// traps: never
/// example: `knuth_break((i, j) => (i + j) / 2, 6) == None`
/// example: `knuth_break((i, j) => if i == 0 and j == 3: 0 else: i + 1, 4) == Some((0, 3))`
pub fn knuth_break(opt: (Int, Int) -> Int, n: Int) -> ?(Int, Int):
    for len in 3..=n:
        for i in 0..=n - len:
            let j = i + len
            let k = opt(i, j)
            if k < opt(i, j - 1) or k > opt(i + 1, j): return Some((i, j))
    None

/// The differences of neighbours, `[xs[i + 1] - xs[i] for i in 0..xs.len - 1]`.
///
/// cost: O(n)
/// empty: `[]` for fewer than 2 elements
/// order: by `i`
/// traps: never (overflow wraps in release, traps in debug)
/// example: `differences([1, 4, 9, 16]) == [3, 5, 7]`
pub fn differences(xs: List<Int>) -> List<Int>:
    [xs[i + 1] - xs[i] for i in 0..xs.len - 1]

/// The Newton forward-difference coefficients of `xs`: element `k` is the first entry of the
/// `k`-th differences, so `xs[x] == sum of newton(xs)[k] * choose(x, k)` for every `x` in
/// `0..xs.len`.
///
/// cost: O(n^2)
/// empty: `[]`
/// order: by `k`, from the 0th differences
/// traps: never (overflow wraps in release, traps in debug)
/// example: `newton([1, 4, 9, 16]) == [1, 3, 2, 0]`
pub fn newton(xs: List<Int>) -> List<Int>:
    var row = xs.copy()
    var out: List<Int> = []
    while not row.is_empty:
        out.push(row[0])
        row = differences(row)
    out

/// The degree of the polynomial `xs` follows at `0, 1, 2, ...`: the smallest `d` whose
/// `(d + 1)`-th differences are all zero, with at least one such difference to confirm it (so
/// `xs.len >= d + 2`); `None` when `xs` is too short to confirm any degree. A zero sequence has
/// degree 0.
///
/// cost: O(n^2)
/// empty: `None` for fewer than 2 elements
/// traps: never (overflow wraps in release, traps in debug)
/// example: `poly_degree([1, 4, 9, 16]) == Some(2)`
/// example: `poly_degree([7, 7]) == Some(0)`
/// example: `poly_degree([1, 2, 4]) == None`
pub fn poly_degree(xs: List<Int>) -> ?Int:
    var row = differences(xs)
    var d = 0
    while not row.is_empty:
        if row.all(x => x == 0): return Some(d)
        row = differences(row)
        d += 1
    None

/// The value at `n` of the polynomial of degree below `xs.len` through `(x, xs[x])` for `x` in
/// `0..xs.len`, from its Newton coefficients; `n` may lie outside `0..xs.len` or be negative.
///
/// cost: O(len^2) for `len = xs.len`
/// empty: `0` when `xs` is empty
/// traps: never (overflow wraps in release, traps in debug)
/// example: `poly_at([1, 4, 9, 16], 10) == 121`
/// example: `poly_at([1, 4, 9, 16], -2) == 1`
pub fn poly_at(xs: List<Int>, n: Int) -> Int:
    let c = newton(xs)
    var total = 0
    var binom = 1
    for k in 0..c.len:
        total += c[k] * binom
        binom = binom * (n - k) / (k + 1)
    total

/// The shortest linear recurrence `xs` satisfies, by Berlekamp-Massey: `c` with
/// `xs[i] == sum of c[j] * xs[i - 1 - j]` for `j` in `0..c.len`, for every `i >= c.len`. Exact
/// over the field `Mod<P>` (`P` prime); with `xs.len >= 2 * c.len` it is the only one that short.
///
/// cost: O(n^2) for `n = xs.len`
/// empty: `[]` (no terms satisfy the empty recurrence; so does an all-zero `xs`)
/// order: `c[0]` multiplies the term just before
/// traps: never
/// example: `type Z = Mod<998244353>; berlekamp_massey([Z(1), Z(1), Z(2), Z(3), Z(5), Z(8)]) == [Z(1), Z(1)]`
/// example: `type Z = Mod<998244353>; berlekamp_massey([Z(0), Z(0)]) == []`
pub fn berlekamp_massey<P>(xs: List<Mod<P>>) -> List<Mod<P>>:
    var cur: List<Mod<P>> = [Mod<P>(1)]
    var prev: List<Mod<P>> = [Mod<P>(1)]
    var len = 0
    var shift = 1
    var last = Mod<P>(1)
    for i in 0..xs.len:
        var d = xs[i]
        for j in 1..=min(len, cur.len - 1): d += cur[j] * xs[i - j]
        if d == Mod<P>(0):
            shift += 1
            continue
        let coef = d / last
        var next = cur.copy()
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
    [Mod<P>(0) - cur[j] for j in 1..=len]

// `p` modulo the characteristic polynomial `x^k - c[0] x^(k-1) - ... - c[k-1]`, as `k` low
// coefficients.
fn reduce_rec<P>(p: List<Mod<P>>, c: List<Mod<P>>) -> List<Mod<P>>:
    let k = c.len
    var q = p.copy()
    while q.len < k: q.push(Mod<P>(0))
    var top = q.len - 1
    while top >= k:
        let t = q[top]
        for j in 0..c.len: q[top - 1 - j] += t * c[j]
        top -= 1
    [q[i] for i in 0..k]

fn mul_rec<P>(a: List<Mod<P>>, b: List<Mod<P>>, c: List<Mod<P>>) -> List<Mod<P>>:
    var p = List.filled(a.len + b.len - 1, Mod<P>(0))
    for i in 0..a.len:
        for j in 0..b.len: p[i + j] += a[i] * b[j]
    reduce_rec(p, c)

/// Term `n` of the sequence with recurrence `c` (as `berlekamp_massey` returns it) and first
/// terms `init`: `init[n]` for `n < c.len`, else by Kitamasa, `x^n` modulo the characteristic
/// polynomial.
///
/// cost: O(k^2 log n) for `k = c.len`
/// empty: `0` when `c` is empty (the recurrence makes every term zero)
/// traps: when `n < 0` or `init.len != c.len`
/// example: `type Z = Mod<998244353>; recurrence_at([Z(1), Z(1)], [Z(0), Z(1)], 10) == Z(55)`
/// example: `type Z = Mod<998244353>; recurrence_at([Z(2)], [Z(3)], 4) == Z(48)`
pub fn recurrence_at<P>(c: List<Mod<P>>, init: List<Mod<P>>, n: Int) -> Mod<P>:
    assert n >= 0, "recurrence_at requires n >= 0"
    assert init.len == c.len, "recurrence_at requires init.len == c.len"
    let k = c.len
    if k == 0: return Mod<P>(0)
    if n < k: return init[n]
    var res = reduce_rec([Mod<P>(1)], c)
    var base = reduce_rec([Mod<P>(0), Mod<P>(1)], c)
    var e = n
    while e > 0:
        if e % 2 == 1: res = mul_rec(res, base, c)
        base = mul_rec(base, base, c)
        e /= 2
    var total = Mod<P>(0)
    for i in 0..k: total += res[i] * init[i]
    total

/// Term `n` of `xs` extended by the shortest recurrence it satisfies, or `None` when `xs` is too
/// short to pin that recurrence: Berlekamp-Massey finds `c`, and the guess stands only when
/// `xs.len > 2 * c.len`, so at least one term past the ones that determine `c` agrees with it.
/// Pass more terms for more confidence. `xs[n]` itself when `n < xs.len`.
///
/// cost: O(xs.len^2 + k^2 log n) for the recurrence length `k`
/// empty: `None` when `xs` is empty
/// traps: when `n < 0`
/// example: `type Z = Mod<998244353>; guess_at([Z(1), Z(1), Z(2), Z(3), Z(5)], 9) == Some(Z(55))`
/// example: `type Z = Mod<998244353>; guess_at([Z(1), Z(2), Z(4), Z(8)], 2) == Some(Z(4))`
/// example: `type Z = Mod<998244353>; guess_at([Z(1), Z(5), Z(2), Z(7)], 10) == None`
pub fn guess_at<P>(xs: List<Mod<P>>, n: Int) -> ?Mod<P>:
    assert n >= 0, "guess_at requires n >= 0"
    if n < xs.len: return Some(xs[n])
    let c = berlekamp_massey(xs)
    if xs.len <= 2 * c.len: return None
    Some(recurrence_at(c, [xs[i] for i in 0..c.len], n))

@test
fn recurrence_tools_agree_with_brute_force():
    type Z = Mod<998_244_353>
    var xs: List<Z> = [Z(2), Z(7), Z(1)]
    for i in 3..40: xs.push(Z(3) * xs[i - 1] - Z(5) * xs[i - 3])
    let c = berlekamp_massey(xs)
    assert c.len <= 3
    for i in c.len..xs.len:
        var s = Z(0)
        for j in 0..c.len: s += c[j] * xs[i - 1 - j]
        assert s == xs[i]
    for n in 0..xs.len: assert recurrence_at(c, [xs[i] for i in 0..c.len], n) == xs[n]
    let head = [xs[i] for i in 0..10]
    for n in 10..40: assert guess_at(head, n) == Some(xs[n])

@test
fn polynomial_tools_agree_with_brute_force():
    let f = (x: Int) => 2 * x * x * x - x + 5
    let xs = [f(x) for x in 0..6]
    assert poly_degree(xs) == Some(3)
    for x in -5..20: assert poly_at(xs, x) == f(x)
    assert poly_degree([f(x) for x in 0..4]) == None
