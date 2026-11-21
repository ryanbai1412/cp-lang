//! Precomputed powers of one base modulo `P`: `pw[k]` in O(1) after O(n) setup.

/// `base ** k` for every `k` in `0..=upto`. Counting problems that multiply by
/// `2 ** (n - k)` in an inner loop read from here rather than calling `pow`.
use std.mod.{Mod}
pub struct Powers<P>(base: Mod<P>):
    pw: List<Mod<P>>

    pub fn new(base: Int, upto: Int) -> Powers<P>:
        let b = Mod<P>(base)
        var pw = [Mod<P>.one]
        for _ in 0..upto: pw.push(pw.last! * b)
        Powers(b, pw)

    /// `base ** k`. Stops if `k` is outside `0..=upto`.
    pub fn get(self, k: Int) -> Mod<P> = pw[k]

    /// `1 + base + ... + base ** (k - 1)`, the geometric sum of `k` terms. O(k) at most once
    /// per call; for one base and many `k`, fold `pw` with `scan` instead.
    pub fn geometric(self, k: Int) -> Mod<P>:
        let acc: Mod<P> = sum(pw[i] for i in 0..k)
        acc

@test
fn powers_match_pow():
    type M = Mod<1_000_000_007>
    let two = Powers<1_000_000_007>(2, 70)
    for k in 0..=70: assert two[k] == M(2) ** k
    assert two[0] == M(1) and two.geometric(10) == M(1023) and two.geometric(0) == M(0)

@test
fn subsets_with_a_given_minimum():
    // the sum over all non-empty subsets of the sorted list of their minimum:
    // xs[i] is the minimum of 2 ** (n - 1 - i) subsets
    type M = Mod<998244353>
    let xs = [1, 3, 4, 9]
    let two = Powers<998244353>(2, xs.len)
    let total: M = sum(M(xs[i]) * two[xs.len - 1 - i] for i in 0..xs.len)
    let want = sum(min(xs[i] for i in 0..4 if (mask >> i) % 2 == 1) for mask in 1..16)
    assert total == M(want)
