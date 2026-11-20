//! Helpers for guessing structure from a slow, obviously correct solution.
//!
//! Compute a table or a sequence with the brute force, then ask:
//! is it monotone, is it convex, does the cost satisfy the quadrangle
//! inequality (so divide-and-conquer or Knuth's optimization applies), is
//! it a polynomial, does it follow a linear recurrence? Each check answers
//! with the first counterexample, `none` meaning "holds on this data".
//! A check that holds is evidence, not a proof.
use std.mod.{Mod}

// ---- shape of a sequence

/// The first `i` with `xs[i] > xs[i + 1]`, or `none` if `xs` never decreases. O(n).
pub fn decrease<T>(xs: List<T>) -> ?Int:
    for i in 0..xs.len - 1:
        if xs[i + 1] < xs[i]: return i
    none

/// The first `i` with `xs[i] < xs[i + 1]`, or `none` if `xs` never increases. O(n).
pub fn increase<T>(xs: List<T>) -> ?Int:
    for i in 0..xs.len - 1:
        if xs[i] < xs[i + 1]: return i
    none

/// The first `i` where convexity breaks: `xs[i + 1] - xs[i] > xs[i + 2] - xs[i + 1]`.
/// `none` means the differences never decrease, so slope tricks and
/// Lagrangian relaxation ("aliens") are worth trying. O(n).
pub fn convexity_break(xs: List<Int>) -> ?Int = decrease(differences(xs))

/// The first `i` where concavity breaks; the mirror image of `convexity_break`.
pub fn concavity_break(xs: List<Int>) -> ?Int = increase(differences(xs))

/// The first `i` where `xs` stops going down and then up: `none` for a
/// sequence that decreases, then increases (flat steps allowed), which is
/// what ternary search needs once flat steps are excluded. O(n).
pub fn unimodality_break(xs: List<Int>) -> ?Int:
    var rising = false
    for i in 0..xs.len - 1:
        if xs[i] < xs[i + 1]: rising = true
        elif xs[i + 1] < xs[i] and rising: return i
    none

// ---- shape of a cost function, for DP optimizations

/// The first `(i, j)` where the quadrangle inequality fails:
/// `cost(i, j) + cost(i + 1, j + 1) > cost(i, j + 1) + cost(i + 1, j)`.
/// `none` over `0..rows` x `0..cols` means `cost` is Monge there, which
/// makes row minima monotone. O(rows * cols) calls.
pub fn monge_break(cost: (Int, Int) -> Int, rows: Int, cols: Int) -> ?(Int, Int):
    for i in 0..rows - 1:
        for j in 0..cols - 1:
            if cost(i, j) + cost(i + 1, j + 1) > cost(i, j + 1) + cost(i + 1, j): return (i, j)
    none

/// For each row, the column of its smallest entry (the leftmost on ties). O(rows * cols) calls.
pub fn argmins(cost: (Int, Int) -> Int, rows: Int, cols: Int) -> List<Int>:
    var out = List<Int>()
    for i in 0..rows:
        var best = 0
        for j in 1..cols:
            if cost(i, j) < cost(i, best): best = j
        out.push(best)
    out

/// The first row whose best column lies left of the previous row's, or
/// `none`: then `dp[i] = min over j of cost(i, j)` can be filled by
/// divide and conquer in O(n log n) instead of O(n^2).
pub fn dnc_break(cost: (Int, Int) -> Int, rows: Int, cols: Int) -> ?Int = decrease(argmins(cost, rows, cols))

/// Knuth's condition on a table of optimal split points for intervals
/// `i..=j` (`opt(i, j)` defined for `i < j`): the first `(i, j)` with
/// `opt(i, j - 1) > opt(i, j)` or `opt(i, j) > opt(i + 1, j)`, or `none`. O(n^2) calls.
pub fn knuth_break(opt: (Int, Int) -> Int, n: Int) -> ?(Int, Int):
    for i in 0..n:
        for j in i + 2..n:
            if opt(i, j - 1) > opt(i, j) or opt(i, j) > opt(i + 1, j): return (i, j)
    none

// ---- finite differences

/// `xs[i + 1] - xs[i]` for every `i`. O(n).
pub fn differences(xs: List<Int>) -> List<Int> = [xs[i + 1] - xs[i] for i in 0..xs.len - 1]

/// The leading entries of the repeated difference rows: `[xs[0], d(xs)[0], d(d(xs))[0], ...]`,
/// cut after the last non-zero one. These are the Newton coefficients of the sequence. O(n^2).
pub fn newton(xs: List<Int>) -> List<Int>:
    var (row, lead) = (copy(xs), List<Int>())
    while not row.is_empty:
        lead.push(row[0])
        row = differences(row)
    while not lead.is_empty and lead.last == 0: lead.pop()
    lead

/// The degree of the polynomial through `xs` (taken at 0, 1, 2, ...), if the
/// data supports one: at least one extra point must confirm it. `none` when
/// `xs` is not visibly a polynomial; the zero sequence has degree -1. O(n^2).
pub fn poly_degree(xs: List<Int>) -> ?Int:
    let d = newton(xs).len - 1
    if d + 1 < xs.len: d else: none

/// The value at any `n >= 0` of the lowest-degree polynomial through `xs`
/// (taken at 0, 1, 2, ...): the sum of `C(n, k) * newton(xs)[k]`. O(len^2).
/// Exact as long as the result fits in an `Int`.
pub fn poly_at(xs: List<Int>, n: Int) -> Int:
    var (total, binom) = (0, 1)
    let lead = newton(xs)
    for k in 0..lead.len:
        total += binom * lead[k]
        binom = binom * (n - k) / (k + 1)
    total

// ---- linear recurrences

/// The shortest linear recurrence of a sequence (Berlekamp-Massey), for a
/// prime `P`: coefficients `c` with `xs[i] == sum of c[j] * xs[i - 1 - j]`
/// for every `i >= c.len`. O(n^2). Trust it when `xs.len >= 2 * c.len + 2`.
pub fn berlekamp_massey<P>(xs: List<Mod<P>>) -> List<Mod<P>>:
    let zero = Mod<P>(0)
    var (cur, last) = (List<Mod<P>>(), List<Mod<P>>())
    var (last_at, last_gap) = (-1, zero)
    for i in 0..xs.len:
        var guess = zero
        for j in 0..cur.len: guess = guess + cur[j] * xs[i - 1 - j]
        let gap = xs[i] - guess
        if gap == zero: continue
        if last_at < 0:
            // the first non-zero term: any recurrence of length i + 1 fits so far
            cur = List.filled(i + 1, zero)
            (last_at, last_gap) = (i, gap)
            continue
        // cancel this gap with a shifted, scaled copy of the recurrence that failed at last_at
        let k = gap / last_gap
        var fix = List.filled(i - last_at - 1, zero)
        fix.push(k)
        for c in last: fix.push(-c * k)
        if fix.len < cur.len: fix.resize(cur.len, zero)
        for j in 0..cur.len: fix[j] = fix[j] + cur[j]
        if i - last_at + last.len >= cur.len:
            (last, last_at, last_gap) = (cur, i, gap)
        cur = fix
    cur

/// Term `n` of the sequence that starts with `init` and continues by the
/// recurrence `c` (as returned by `berlekamp_massey`). O(k^2 log n) for `k = c.len`.
pub fn recurrence_at<P>(c: List<Mod<P>>, init: List<Mod<P>>, n: Int) -> Mod<P>:
    let k = c.len
    let zero = Mod<P>(0)
    if n < init.len: return init[n]
    if k == 0: return zero
    // x^n modulo the characteristic polynomial, by repeated squaring
    var acc = List.filled(k, zero)
    var base = List.filled(k, zero)
    acc[0] = Mod<P>(1)
    if k == 1: base[0] = c[0] else: base[1] = Mod<P>(1)
    var e = n
    while e > 0:
        if e % 2 == 1: acc = mul_mod_char(acc, base, c)
        base = mul_mod_char(base, base, c)
        e /= 2
    var total = zero
    for i in 0..k: total = total + acc[i] * init[i]
    total

/// The product of two polynomials of degree below `k`, reduced by `x^k = sum of c[j] * x^(k - 1 - j)`.
fn mul_mod_char<P>(a: List<Mod<P>>, b: List<Mod<P>>, c: List<Mod<P>>) -> List<Mod<P>>:
    let k = c.len
    var prod = List.filled(2 * k - 1, Mod<P>(0))
    for i in 0..k:
        for j in 0..k: prod[i + j] = prod[i + j] + a[i] * b[j]
    for d in (k..2 * k - 1).rev:
        for j in 0..k: prod[d - 1 - j] = prod[d - 1 - j] + prod[d] * c[j]
    prod[0..k]

/// Guess term `n` of a sequence from its first terms, assuming a linear recurrence.
/// `none` when the data is too short to confirm the recurrence it finds.
pub fn guess_at<P>(xs: List<Mod<P>>, n: Int) -> ?Mod<P>:
    let c = berlekamp_massey(xs)
    if xs.len < 2 * c.len + 2: return none
    recurrence_at(c, xs, n)

@test
fn sequence_shapes():
    assert decrease([1, 2, 2, 5]) == none and decrease([1, 3, 2, 5]) == 1
    assert increase([5, 5, 1]) == none and increase([5, 6, 1]) == 0
    assert decrease(List<Int>()) == none and decrease([7]) == none
    let squares = [x * x for x in -5..=5]
    assert convexity_break(squares) == none and concavity_break(squares) == 0
    assert convexity_break([0, 3, 4, 9]) == 0
    assert unimodality_break(squares) == none
    assert unimodality_break([5, 3, 4, 2, 6]) == 2

@test
fn monge_costs_have_monotone_row_minima():
    let sq = (i, j) => (i - j) * (i - j)
    assert monge_break(sq, 9, 9) == none
    assert dnc_break(sq, 9, 9) == none
    assert argmins(sq, 4, 4) == [0, 1, 2, 3]
    // random Monge matrices: sums of a random non-negative density over a corner, negated
    for _ in 0..20:
        let w = [[rand(0, 5) for _ in 0..7] for _ in 0..7]
        let cost = (i, j) => rand_free(i, j) - sum(w[a][b] for a in 0..=i for b in 0..=j)
        assert monge_break(cost, 7, 7) == none
        assert dnc_break(cost, 7, 7) == none
    let bad = (i, j) => if i == j: 5 else: 0
    assert monge_break(bad, 3, 3) == (0, 0)
    assert dnc_break((i, j) => if i + j == 3: 0 else: 1, 4, 4) == 0

/// A row term plus a column term: adding it keeps a matrix Monge.
fn rand_free(i: Int, j: Int) -> Int = 3 * i - 2 * j

@test
fn knuth_condition_on_optimal_merge_splits():
    // merging adjacent piles: cost of an interval is its sum, a classic Knuth-optimizable DP
    let w = [rand(1, 9) for _ in 0..9]
    let n = w.len
    var dp = [[0 for _ in 0..n] for _ in 0..n]
    var opt = [[i for _ in 0..n] for i in 0..n]
    for len in 2..=n:
        for i in 0..=n - len:
            let j = i + len - 1
            var best = Int.MAX
            for k in i..j:
                let c = dp[i][k] + dp[k + 1][j]
                if c < best:
                    best = c
                    opt[i][j] = k
            dp[i][j] = best + sum(w[t] for t in i..=j)
    assert knuth_break((i, j) => opt[i][j], n) == none
    assert knuth_break((i, j) => -j, n) == (0, 2)

@test
fn finite_differences_find_polynomials():
    let cubes = [x * x * x - 4 * x + 7 for x in 0..9]
    assert differences([1, 4, 9, 16]) == [3, 5, 7]
    assert poly_degree(cubes) == 3
    assert poly_degree([5, 5, 5]) == 0 and poly_degree([0, 0, 0]) == -1
    assert poly_degree([1, 2, 4, 8, 16, 32]) == none
    assert poly_degree([3, 1, 4]) == none
    for n in [0, 5, 9, 20, 1000]: assert poly_at(cubes, n) == n * n * n - 4 * n + 7
    // sums of fourth powers are a polynomial of degree 5
    var sums = [0]
    for x in 1..10: sums.push(sums.last + x ** 4)
    assert poly_degree(sums) == 5
    assert poly_at(sums, 100) == sum(x ** 4 for x in 1..=100)

@test
fn berlekamp_massey_recovers_recurrences():
    type M = Mod<998244353>
    var fib = [M(0), M(1)]
    for i in 2..12: fib.push(fib[i - 1] + fib[i - 2])
    assert berlekamp_massey(fib) == [M(1), M(1)]
    assert recurrence_at([M(1), M(1)], fib, 90) == M(2880067194370816120)
    assert guess_at(fib, 90)! == M(2880067194370816120)
    assert guess_at(fib[0..5], 90) == none
    assert berlekamp_massey([M(0), M(0), M(0)]) == [] and guess_at([M(0), M(0)], 7) == M(0)
    assert berlekamp_massey([M(3), M(6), M(12), M(24)]) == [M(2)]
    for _ in 0..25:
        let k = rand(1, 4)
        let c = [M(rand(0, 6)) for _ in 0..k]
        var xs = [M(rand(0, 9)) for _ in 0..k]
        for i in k..40:
            var t = M(0)
            for j in 0..k: t = t + c[j] * xs[i - 1 - j]
            xs.push(t)
        let found = berlekamp_massey(xs[0..20])
        assert found.len <= k
        for i in found.len..40:
            var t = M(0)
            for j in 0..found.len: t = t + found[j] * xs[i - 1 - j]
            assert t == xs[i]
        for n in [0, 3, 19, 39]: assert recurrence_at(found, xs[0..20], n) == xs[n]
