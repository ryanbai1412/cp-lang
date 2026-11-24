// expect: ok, output "ok"
// input: "1 300\n"
// model: Sieve
// model: factor
// model: divisors
// model: divisors_from_factors
// model: each_divisor
// model: is_prime
// model: phi
// model: primes_in
// model: prime_count
// model: sieve_multiplicative
// model: carmichael
// model: mult_order
// model: jacobi
// model: primitive_root
// std.math's primes and multiplicative functions against trial division and counting loops:
// a Sieve up to 300 and its tables, then seeded random queries, each compared with the loop.
// Reads `seed n`; prints the first op that differs.
use std.math.{Sieve, factor, divisors_from_factors, each_divisor, is_prime, phi, primes_in, prime_count}
use std.math.{sieve_multiplicative, carmichael, mult_order, jacobi, primitive_root}
use std.rand.{Rng}

const N = 300

fn trial_prime(x: Int) -> Bool:
    if x < 2: return false
    var d = 2
    while d * d <= x:
        if x % d == 0: return false
        d += 1
    true

fn trial_factor(x: Int) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    var y = x
    var d = 2
    while d * d <= y:
        var e = 0
        while y % d == 0:
            y /= d
            e += 1
        if e > 0: out.push((d, e))
        d += 1
    if y > 1: out.push((y, 1))
    out

fn all_divisors(x: Int) -> List<Int>:
    var out: List<Int> = []
    var d = 1
    while d * d <= x:
        if x % d == 0:
            out.push(d)
            if d * d != x: out.push(x / d)
        d += 1
    out.sorted()

// Counted for small `x`; above that, the product formula over the trial factorization.
fn coprime_count(x: Int) -> Int:
    if x <= 3000: return len([i for i in 1..=x if gcd(i, x) == 1])
    var r = x
    for (p, _) in trial_factor(x): r = r / p * (p - 1)
    r

fn mobius_of(x: Int) -> Int:
    let f = trial_factor(x)
    if any(e > 1 for (_, e) in f): 0 elif f.len % 2 == 0: 1 else: -1

fn order(a: Int, m: Int) -> ?Int:
    if gcd(a, m) != 1: return None
    var (p, k) = (a % m, 1)
    while p != 1 % m:
        p = p * a % m
        k += 1
    Some(k)

fn euler(a: Int, p: Int) -> Int:
    var r = 1
    for _ in 0..(p - 1) / 2: r = r * a % p
    if r == p - 1: -1 else: r

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
let s = Sieve(N)
let (phis, mus, dcs, dss) = (s.phi_table(), s.mobius_table(), s.div_count_table(), s.div_sum_table())
let mult_d = sieve_multiplicative(N, (p, e, pe) => e + 1)
let mult_phi = sieve_multiplicative(N, (p, e, pe) => pe - pe / p)
var bad = ""
if s.primes != [p for p in 2..=N if trial_prime(p)] or s.spf[0] != 0 or s.spf[1] != 0:
    bad = "op -1: primes"
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 8)
    let x = rng.int(1, N)
    if op == 0:
        let ds = all_divisors(x)
        if s.is_prime(x) != trial_prime(x) or s.is_prime(0): bad = "op {k}: Sieve.is_prime({x})"
        elif s.factor(x) != trial_factor(x): bad = "op {k}: Sieve.factor({x})"
        elif s.spf[x] != (if x == 1: 0 else: trial_factor(x)[0].0): bad = "op {k}: spf[{x}]"
        elif s.phi(x) != coprime_count(x) or phis[x] != coprime_count(x): bad = "op {k}: phi({x})"
        elif s.mobius(x) != mobius_of(x) or mus[x] != mobius_of(x): bad = "op {k}: mobius({x})"
        elif dcs[x] != ds.len or dss[x] != sum(ds): bad = "op {k}: divisor tables at {x}"
        elif s.divisors(x).collect().sorted() != ds: bad = "op {k}: Sieve.divisors({x})"
        elif mult_d[x] != ds.len or mult_phi[x] != coprime_count(x): bad = "op {k}: sieve_multiplicative at {x}"
    elif op == 1:
        // a product of up to three random factors, so large primes and repeats both appear
        var y = 1
        for i in 0..rng.int(1, 3):
            y *= if i == 2 or rng.chance(0.5): rng.int(1, 40) else: rng.int(1, 3000)
        let f = trial_factor(y)
        if factor(y) != f: bad = "op {k}: factor({y})"
        elif y.divisors() != all_divisors(y) or divisors_from_factors(f) != all_divisors(y): bad = "op {k}: divisors({y})"
        elif each_divisor(y).collect().sorted() != all_divisors(y): bad = "op {k}: each_divisor({y})"
        elif phi(y) != coprime_count(y): bad = "op {k}: phi({y})"
    elif op == 2:
        let y = if rng.chance(0.5): rng.int(-10, 2000) else: rng.int(1, 3_000_000)
        if is_prime(y) != trial_prime(y): bad = "op {k}: is_prime({y})"
    elif op == 3:
        let p = rng.int(2, 50000)
        let q = rng.int(2, 50000)
        let pq = p * q
        let pp = 1_000_000_007 * (if rng.chance(0.5): 1 else: p)
        if is_prime(pq) or is_prime(pp) != (pp == 1_000_000_007): bad = "op {k}: is_prime of a product"
        elif factor(pq) != trial_factor(pq): bad = "op {k}: factor({pq})"
    elif op == 4:
        let lo = rng.int(-5, 3000)
        let hi = lo + rng.int(-5, 100)
        if primes_in(lo, hi) != [p for p in max(lo, 0)..max(hi, 0) if trial_prime(p)]: bad = "op {k}: primes_in({lo}, {hi})"
        elif prime_count(hi) != len([p for p in 0..=max(hi, 0) if trial_prime(p)]): bad = "op {k}: prime_count({hi})"
    elif op == 5:
        let a = rng.int(-50, 50)
        if mult_order(a, x) != order(((a % x) + x) % x, x): bad = "op {k}: mult_order({a}, {x})"
        else:
            var lam = 1
            for b in 1..=x:
                if order(b, x) != None: lam = lcm(lam, order(b, x)!)
            if carmichael(x) != lam: bad = "op {k}: carmichael({x})"
    elif op == 6:
        let m = 2 * rng.int(0, 150) + 1
        let a = rng.int(-100, 100)
        var want = 1
        for (p, e) in trial_factor(m):
            for _ in 0..e: want *= euler(((a % p) + p) % p, p)
        if jacobi(a, m) != want: bad = "op {k}: jacobi({a}, {m})"
    elif op == 7:
        let p = s.primes[rng.int(0, s.primes.len - 1)]
        var g = 1
        while order(g, p) != Some(p - 1): g += 1
        if primitive_root(p) != g: bad = "op {k}: primitive_root({p})"
    elif op == 8:
        let p = rng.int(2, 20000)
        let ds = all_divisors(p)
        if p.divisors() != ds or each_divisor(p).collect().sorted() != ds: bad = "op {k}: divisors({p})"
print(if bad == "": "ok" else: bad)
