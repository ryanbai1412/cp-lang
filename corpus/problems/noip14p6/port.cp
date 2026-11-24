// Source: NOIP 2014 Senior D2T3, Solve the Equation (not on DMOJ; Luogu P2312)
// limits: 1000 ms, 128 MB
// Problem: given n <= 100, m < 1e6 and integers a_0..a_n with |a_i| <= 10^10000, a_n != 0,
//   print the number of integer roots of a_0 + a_1 x + ... + a_n x^n = 0 in 1..=m, then
//   the roots in increasing order, one per line.
// Solution: f(x) = f(x mod p) (mod p), so under a small prime p one Horner pass over the
//   residues 0..p (`eval`) says which x can be roots; f mod p has at most n roots, so each
//   prime keeps at most n / p of the x. Every x that survives three primes near 10^4 is then
//   checked by Horner modulo 2^61 - 1, which a nonzero value passes only by bad luck (about
//   n / 2^61 per x). O(n (p + m n / p)) instead of O(n m).
// original: corpus/submissions/5445671.cpp
// Bounds: n: 1..=100; m: 1..=1e6
use std.math.{primes_in}

let n: 1..=100 = read()
let m: 1..=1e6 = read()
let digits = read<[0..=n]Str>()

// which residues of x mod p make f vanish modulo p
fn roots_mod(p: Int) -> List<Bool>:
    type Z = Mod(p)
    let under_p = [s.parse<Z>()! for s in digits]
    [under_p.eval(Z(r)) == Z(0) for r in 0..p]

let sieves = [(p:, root: roots_mod(p)) for p in primes_in(10_000, 10_100)[..3]]
type Z = Mod<2305843009213693951>
let coef = [s.parse<Z>()! for s in digits]

let roots = [x for x in 1..=m if sieves.all(s => s.root[x % s.p]) and coef.eval(Z(x)) == Z(0)]
print(roots.len)
for x in roots: print(x)
