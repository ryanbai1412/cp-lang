// NOIP 2014 Solving Equations: integer roots in 1..=m of a polynomial whose coefficients have
// up to 10^4 digits. f(x) = f(x mod p) (mod p), so one pass over a small prime's residues
// rules out most x; each survivor is checked modulo a large prime, where a root vanishes.
use std.math.{primes_in}
let n: 1..=100 = read()
let m: 1..=1e6 = read()
let digits = read<[n + 1]Str>()
fn vanishes_mod(p: Int) -> List<Bool>:
    type Zp = Mod(p)
    let f = [s.parse<Zp>()! for s in digits]
    [f.eval(Zp(r)) == Zp(0) for r in 0..p]
let sieves = [(p:, zero: vanishes_mod(p)) for p in primes_in(10_000, 10_100)[..3]]
type Z = Mod<2305843009213693951>
let f = [s.parse<Z>()! for s in digits]
let roots = [x for x in 1..=m if sieves.all(s => s.zero[x % s.p]) and f.eval(Z(x)) == Z(0)]
print(roots.len)
print(roots, sep: "\n")
