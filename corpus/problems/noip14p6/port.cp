// Source: NOIP 2014 Senior D2T3, Solve the Equation (not on DMOJ; Luogu P2312)
// Problem: given n <= 100, m < 1e6 and integers a_0..a_n with |a_i| <= 10^10000, a_n != 0,
//   print the number of integer roots of a_0 + a_1 x + ... + a_n x^n = 0 in 1..=m, then
//   the roots in increasing order, one per line.
// Solution: read each coefficient modulo two primes, 1e9+7 and 998244353, and test every x
//   in 1..=m by Horner's rule; x is reported when the value is 0 modulo both, which a
//   nonzero value passes only by bad luck. O(n m) per prime.
// original: corpus/submissions/5445671.cpp
let n: 1..=100 = read()
let m: 1..=1e6 = read()

// a short signed number, for stress tests
fn random_number() -> List<Char>:
    var s: List<Char> = []
    if rand(0, 1) == 0: s.push('-')
    for _ in 0..rand(1, 3): s.push('0' + rand(0, 9))
    s

// The numbers are far too long to hold, so keep each one modulo two primes.
// A root is a root modulo both; the other way round is a safe bet.
let primes = [1000000007, 998244353]
var a: [0..2, 0..=n]Int = 0
for i in 0..=n:
    let s: [domain S]Char = read(gen: random_number)
    for t in 0..2:
        var v = 0
        for c in s:
            if c != '-': v = (v * 10 + (c - '0')) % primes[t]
        a[t, i] = if s[S.first!] == '-': -v % primes[t] else: v

fn is_root(x: Int, t: Int) -> Bool:
    var y = 0
    for i in (0..=n).rev: y = (y * x + a[t, i]) % primes[t]  // Horner
    y == 0

let roots = [x for x in 1..=m if is_root(x, 0) and is_root(x, 1)]
print(roots.len)
for x in roots: print(x)
