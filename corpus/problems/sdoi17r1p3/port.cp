// Source: SDOI 2017 Round 1, Sequence Counting (Luogu P3702), https://dmoj.ca/problem/sdoi17r1p3
// Problem: count length-n sequences of integers in 1..=m whose sum is a multiple of p and that
//   contain at least one prime, modulo 20170408 (n <= 1e9, m <= 2e7, p <= 100).
// Solution: all sequences with sum 0 mod p, minus those with no prime. Each count is entry 0 of
//   the n-th power of the vector of residue counts (of 1..=m, or of its non-primes) under
//   cyclic convolution mod p: `pow` by squaring in O(p^2 log n), after a segmented sieve.
// original: corpus/submissions/5463991.cpp
// Bounds: n: 1..=1e9; m: 1..=2e7; p: 1..=100
use std.math.{primes_in}

type Z = Mod<20170408>

let (n, m, p): (1..=1e9, 1..=2e7, 1..=100) = read()
domain Residue = 0..p
type Counts = [Residue]Z

fn convolve(a: Counts, b: Counts) -> Counts:
    var c: Counts = 0
    for i in Residue:
        for j in Residue: c[Residue((i.idx + j.idx) % p)] += a[i] * b[j]
    c

let cyclic = Monoid<Counts>([Residue] r => Z(if r.idx == 0: 1 else: 0), convolve)
// m / p whole cycles hold every residue once; the rest are 1..=m % p
let all: Counts = [Residue] r => Z(m / p + (if 1 <= r.idx and r.idx <= m % p: 1 else: 0))
var plain = all.clone()
for q in primes_in(2, m + 1): plain[Residue(q % p)] -= 1
let zero = Residue(0)
print(pow(all, n, op: cyclic)[zero] - pow(plain, n, op: cyclic)[zero])
