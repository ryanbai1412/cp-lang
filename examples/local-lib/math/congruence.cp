//! Systems of congruences: the Chinese remainder theorem for moduli that need not be
//! coprime, kept as one value that is folded over the system.
use std.math.{ext_gcd, mul_mod, ceil_div}
use std.math.{ext_gcd, mul_mod}

/// All `x` with `x % m == r`, where `m >= 1` and `0 <= r < m`. Combining two is
/// O(log m); nothing overflows as long as the combined modulus fits in an `Int`.
pub struct Congruence(r: Int, m: Int):
    /// `x = r (mod m)`, with `r` of any sign reduced into `0..m`.
    pub fn new(r: Int, m: Int) -> Congruence:
        assert m >= 1
        Congruence(r % m, m)

    /// Every integer: the system with no equations, and the identity of `meet`.
    pub fn any() -> Congruence = Congruence(0, 1)

    /// The `x` with `a * x = b (mod m)`, or `none` when `gcd(a, m)` does not divide `b`.
    pub fn solve(a: Int, b: Int, m: Int) -> ?Congruence:
        assert m >= 1
        let e = ext_gcd(a % m, m)
        if b % e.g != 0: return none
        let step = m / e.g
        Congruence(mul_mod(b % m / e.g % step, e.x % step, step), step)

    /// The `x` that satisfy both, modulo `lcm(m, o.m)`, or `none` when they contradict.
    pub fn meet(self, o: Congruence) -> ?Congruence:
        let e = ext_gcd(m, o.m)
        if (o.r - r) % e.g != 0: return none
        let step = o.m / e.g
        // x = r + m * k with k = (o.r - r) / g * inv(m / g) mod (o.m / g)
        let k = mul_mod((o.r - r) / e.g % step, e.x % step, step)
        Congruence(r + m * k, m * step)

    /// The whole system at once, or `none` at the first contradiction.
    pub fn all(parts: List<Congruence>) -> ?Congruence:
        var acc = Congruence.any()
        for p in parts:
            let next = acc.meet(p)
            if next == none: return none
            acc = next!
        acc

    pub fn holds(self, x: Int) -> Bool = x % m == r

    /// The smallest solution that is `>= at_least`.
    pub fn first(self, at_least: Int = 0) -> Int = r + m * ceil_div(at_least - r, m)

    /// How many solutions lie in `lo..=hi`.
    pub fn count(self, lo: Int, hi: Int) -> Int:
        if lo > hi: return 0
        (hi - r) / m - (lo - 1 - r) / m

@test
fn meet_matches_scanning_one_period():
    for _ in 0..300:
        let (a, b) = (
            Congruence.new(rand(-40, 40), rand(1, 24)),
            Congruence.new(rand(-40, 40), rand(1, 24)),
        )
        let hits = [x for x in 0..a.m * b.m if a.holds(x) and b.holds(x)]
        let got = a.meet(b)
        if hits.is_empty:
            assert got == none
        else:
            // the solutions are one residue class modulo the lcm
            assert got!.r == hits[0] and got!.m * hits.len == a.m * b.m
            assert got == b.meet(a)

@test
fn linear_congruences_match_brute_force():
    for _ in 0..300:
        let (a, b, m) = (rand(-30, 30), rand(-30, 30), rand(1, 30))
        let hits = [x for x in 0..m if (a * x - b) % m == 0]
        let got = Congruence.solve(a, b, m)
        if hits.is_empty:
            assert got == none
        else:
            assert hits == [x for x in 0..m if got!.holds(x)]

@test
fn a_whole_system_and_its_first_solution():
    for _ in 0..200:
        let hidden = rand(-500, 500)
        let parts = [Congruence.new(hidden, rand(1, 12)) for _ in 0..rand(0, 5)]
        // a system built from one hidden number always has it as a solution
        let sys = Congruence.all(parts)!
        assert sys.holds(hidden)
        let from = rand(-600, 600)
        let x = sys.first(at_least: from)
        assert x >= from and x - sys.m < from and parts.all(p => p.holds(x))
        let (lo, hi) = (rand(-80, 80), rand(-80, 80))
        assert sys.count(lo, hi) == (lo..=hi).count(y => sys.holds(y))
    assert Congruence.all([Congruence.new(1, 4), Congruence.new(2, 6)]) == none
    assert Congruence.any().first(at_least: -7) == -7

@test
fn large_moduli_do_not_overflow():
    // three primes near 2 * 10^6: the combined modulus is about 8 * 10^18
    let ms = [1999993, 2000003, 2000029]
    let x = 7_654_321_987_654_321
    let sys = Congruence.all([Congruence.new(x, m) for m in ms])!
    assert sys.m == ms[0] * ms[1] * ms[2] and sys.r == x

@test
fn when_three_gears_line_up():
    // gear i shows its mark every period[i] ticks, first at tick start[i]
    let period = [4, 6, 9]
    let start = [3, 1, 7]
    let sys = Congruence.all([Congruence.new(start[i], period[i]) for i in 0..3])!
    var t = 7
    while not (0..3).all(i => (t - start[i]) % period[i] == 0): t += 1
    assert sys.first(at_least: 7) == t and sys.m == 36
