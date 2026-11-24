// Source: SDOI 2017 Round 1, Sequence Counting (Luogu P3702), https://dmoj.ca/problem/sdoi17r1p3
// Problem: count length-n sequences of integers in 1..m whose sum is a multiple of p and that
//   contain at least one prime, modulo 20170408 (n <= 1e9, m <= 2e7, p <= 100).
// Solution: Count all sequences with sum = 0 mod p and subtract those with no prime. Each
//   count is entry 0 of the n-th power of the vector of residue counts (of 1..m, or of the
//   non-primes, found by a sieve) under cyclic convolution mod p: binary exponentiation in
//   O(p^2 log n) after an O(m log log m) sieve. The port's own bounds are m <= 200, p <= 20.
// original: corpus/submissions/5463991.cpp
const P = 20170408
type Z = Mod<P>

let n: 1..=1e9 = read()
let m: 1..=200 = read()  // the statement allows 2e7; kept small for the checker
let k: 1..=20 = read()   // the statement allows 100
domain R = 0..k
type Vec = [R]Z

// c[(i + j) mod k] += a[i] * b[j]
fn conv(a: Vec, b: Vec) -> Vec:
    var c: Vec = 0
    for i in R:
        for j in R: c[R((i.idx + j.idx) % k)] += a[i] * b[j]
    c

// x convolved with itself e times, e >= 1
fn power(x: Vec, e: Int) -> Vec:
    var base = x
    var acc = x
    var t = e - 1
    while t > 0:
        if t % 2 == 1: acc = conv(acc, base)
        base = conv(base, base)
        t /= 2
    acc

var composite: [0..=m]Bool = false
for i in 2..=m:
    if not composite[i]:
        var j = 2 * i
        while j <= m:
            composite[j] = true
            j += i

var all: Vec = 0
var plain: Vec = 0
for x in 1..=m:
    all[R(x % k)] += 1
    if x == 1 or composite[x]: plain[R(x % k)] += 1
print(power(all, n)[R(0)] - power(plain, n)[R(0)])
