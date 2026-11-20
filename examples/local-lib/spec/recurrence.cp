//! Linear recurrences found from the first terms of a brute force, over the rationals and
//! modulo a prime. `std.spec` holds Berlekamp-Massey for `Mod<P>`; this module adds the
//! exact search (so a recurrence with small integer coefficients is seen as such, not as
//! residues), a confirmation rule, and the n-th term of an integer recurrence.
use std.mod.{Mod}
use std.spec.{berlekamp_massey, recurrence_at}
use lib.math.frac.{Frac}

/// The shortest linear recurrence of `xs` over the rationals (Berlekamp-Massey with exact
/// fractions): coefficients `c` with `xs[i] == sum of c[j] * xs[i - 1 - j]` for every
/// `i >= c.len`. O(n^2) fraction operations; the fractions stay small when the data
/// really follows a short recurrence, and overflow stops the program otherwise.
pub fn rational_recurrence(xs: List<Int>) -> List<Frac>:
    let zero = Frac.new(0)
    var (cur, last) = (List<Frac>(), List<Frac>())
    var (last_at, last_gap) = (-1, zero)
    for i in 0..xs.len:
        var guess = zero
        for j in 0..cur.len: guess = guess + cur[j] * Frac.new(xs[i - 1 - j])
        let gap = Frac.new(xs[i]) - guess
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
        for c in last: fix.push(zero - c * k)
        if fix.len < cur.len: fix.resize(cur.len, zero)
        for j in 0..cur.len: fix[j] = fix[j] + cur[j]
        if i - last_at + last.len >= cur.len:
            (last, last_at, last_gap) = (cur, i, gap)
        cur = fix
    cur

/// The shortest recurrence of `xs` when it is one a contestant can use: every coefficient
/// a whole number, and at least two terms beyond the `2 * c.len` that determine it agree.
/// `none` when the data is too short to confirm it or the coefficients are fractions
/// (look at `rational_recurrence` then).
pub fn integer_recurrence(xs: List<Int>) -> ?List<Int>:
    let c = rational_recurrence(xs)
    if xs.len < 2 * c.len + 2: return none
    for f in c:
        if f.den != 1: return none
    [f.num for f in c]

/// The shortest recurrence of `xs` modulo the prime `P`, or `none` when the data is too
/// short to confirm it. Use it when the brute force already prints residues, or when the
/// exact coefficients are fractions.
pub fn mod_recurrence<P>(xs: List<Int>) -> ?List<Mod<P>>:
    let c = berlekamp_massey([Mod<P>(x) for x in xs])
    if xs.len < 2 * c.len + 2: none else: c

/// The first `i >= c.len` where `xs[i]` is not `sum of c[j] * xs[i - 1 - j]`, or `none`
/// when `xs` follows the recurrence `c` throughout. O(n * c.len).
pub fn recurrence_break(c: List<Int>, xs: List<Int>) -> ?Int:
    for i in c.len..xs.len:
        if xs[i] != sum(c[j] * xs[i - 1 - j] for j in 0..c.len): return i
    none

/// The first `n` terms of the sequence that starts with `init` and continues by `c`.
/// Exact, so only for terms that fit in an `Int`. O(n * c.len).
pub fn extend(c: List<Int>, init: List<Int>, n: Int) -> List<Int>:
    assert init.len >= c.len
    var xs = [init[i] for i in 0..min(n, init.len)]
    for i in xs.len..n: xs.push(sum(c[j] * xs[i - 1 - j] for j in 0..c.len))
    xs

/// Term `n` modulo `P` of the sequence that starts with `init` and continues by the
/// integer recurrence `c` (Kitamasa: `x^n` reduced by the characteristic polynomial).
/// O(k^2 log n) for `k = c.len`, so `n = 10^18` is fine.
pub fn nth_term<P>(c: List<Int>, init: List<Int>, n: Int) -> Mod<P>:
    assert init.len >= c.len
    if n < init.len: return Mod<P>(init[n])
    if c.is_empty: return Mod<P>(0)
    recurrence_at([Mod<P>(x) for x in c], [Mod<P>(x) for x in init], n)

/// The recurrence written out, to paste into a comment: `a[n] = 4 a[n-1] - a[n-2]`.
pub fn show_recurrence(c: List<Int>) -> Str:
    var out = "a[n] ="
    var any_term = false
    for j in 0..c.len:
        if c[j] == 0: continue
        let mag = abs(c[j])
        if c[j] < 0: out = out + " -" elif any_term: out = out + " +"
        out += " "
        if mag != 1: out = out + "{mag} "
        out += "a[n-{j + 1}]"
        any_term = true
    if any_term: out else: out + " 0"

@test
fn exact_search_finds_small_integer_coefficients():
    var fib = [0, 1]
    for i in 2..14: fib.push(fib[i - 1] + fib[i - 2])
    assert rational_recurrence(fib) == [Frac(1, 1), Frac(1, 1)]
    assert integer_recurrence(fib) == [1, 1]
    // five terms fix a recurrence of length 2 but leave only one to confirm it
    assert integer_recurrence(fib[0..5]) == none and integer_recurrence(fib[0..6]) == [1, 1]
    assert integer_recurrence([3, 6, 12, 24]) == [2]
    assert integer_recurrence([0, 0, 0]) == [] and integer_recurrence([7, 7, 7, 7]) == [1]
    // negative and zero coefficients: a[n] = 4 a[n-1] - a[n-2], and a[n] = a[n-3] - 2 a[n-1]
    assert integer_recurrence(extend([4, -1], [1, 3], 10)) == [4, -1]
    assert integer_recurrence(extend([-2, 0, 1], [1, 0, 2], 12)) == [-2, 0, 1]

@test
fn fractions_are_reported_not_rounded():
    // each term is the average of the two before it: a[n] = a[n-1] / 2 + a[n-2] / 2
    let exact = [0, 16, 8, 12, 10, 11]
    assert rational_recurrence(exact) == [Frac(1, 2), Frac(1, 2)]
    assert integer_recurrence(exact) == none
    type M = Mod<998244353>
    let half = M(1) / M(2)
    assert mod_recurrence::<998244353>(exact) == [half, half]

@test
fn random_recurrences_are_recovered_and_continued():
    type M = Mod<1000000007>
    for _ in 0..40:
        let k = rand(1, 4)
        let c = [rand(-3, 3) for _ in 0..k]
        let init = [rand(-4, 4) for _ in 0..k]
        let xs = extend(c, init, 2 * k + 8)
        assert recurrence_break(c, xs) == none
        let found = rational_recurrence(xs)
        assert found.len <= k
        // whatever was found reproduces the data, and the mod-P search agrees on the length
        for i in found.len..xs.len:
            var t = Frac.new(0)
            for j in 0..found.len: t = t + found[j] * Frac.new(xs[i - 1 - j])
            assert t == Frac.new(xs[i])
        assert mod_recurrence::<1000000007>(xs)!.len == found.len
        if let ints = integer_recurrence(xs):
            assert recurrence_break(ints, xs) == none
            for n in [0, 1, k, 2 * k + 7]: assert nth_term::<1000000007>(ints, xs, n) == M(xs[n])

@test
fn a_wrong_guess_is_located():
    let squares = [x * x for x in 0..8]
    assert recurrence_break([2, -1], squares) == 2
    assert recurrence_break([3, -3, 1], squares) == none
    assert integer_recurrence(squares) == [3, -3, 1]
    assert recurrence_break([1, 1], [1, 1, 2, 3, 5, 9]) == 5
    assert extend([1, 1], [1, 1], 0) == [] and extend([1, 1], [1, 1, 2], 1) == [1]

@test
fn the_nth_term_reaches_far():
    type M = Mod<1000000007>
    // Fibonacci number 10^18 modulo 10^9 + 7
    assert nth_term::<1000000007>([1, 1], [0, 1], 1000000000000000000) == M(209783453)
    assert nth_term::<1000000007>([2], [1], 100) == M(2).pow(100)
    assert nth_term::<1000000007>([], [], 5) == M(0)
    let c = [4, -1]
    let xs = extend(c, [1, 3], 30)
    for n in 0..30: assert nth_term::<1000000007>(c, [1, 3], n) == M(xs[n])

@test
fn recurrences_print_the_way_they_are_written():
    assert show_recurrence([1, 1]) == "a[n] = a[n-1] + a[n-2]"
    assert show_recurrence([4, -1]) == "a[n] = 4 a[n-1] - a[n-2]"
    assert show_recurrence([-2, 0, 1]) == "a[n] = - 2 a[n-1] + a[n-3]"
    assert show_recurrence([]) == "a[n] = 0" and show_recurrence([0, -1]) == "a[n] = - a[n-2]"
