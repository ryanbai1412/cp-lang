// Source: SDOI 2017 Round 1, Number Table (Luogu P3704), https://dmoj.ca/problem/sdoi17r1p1
// Problem: multitest (1 <= T <= 1000): given 1 <= n, m <= 1e6, print the product of f(gcd(i, j)) over
//   1 <= i <= n, 1 <= j <= m, where f is Fibonacci (f0 = 0, f1 = 1), modulo 1e9+7.
// Solution: Group by d = gcd and apply Mobius inversion: with g(T) = product over d | T of
//   f(d)^mu(T/d), the answer is the product over T of g(T)^((n/T)(m/T)). Sieve mu, build g by
//   a harmonic loop in O(N log N), keep prefix products of g and their inverses; per query
//   n/T and m/T are constant on O(sqrt n) blocks, one power each: O(sqrt(n) log P).
// original: corpus/submissions/5463988.cpp

use std.prefix.{prefix_fold}

type Z = Mod<1_000_000_007>

let t: 1..=1000 = read()
let asks: [t](n: 1..=1e6, m: 1..=1e6) = read()
let top = max(max(a.n, a.m) for a in asks)!
let mu = Sieve(top).mobius_table()

var fib = List.filled(top + 1, Z.zero)
fib[1] = Z.one
for i in 2..=top: fib[i] = fib[i - 1] + fib[i - 2]

// g[T] = product over d | T of fib[d]^mu(T / d)
var g = List.filled(top + 1, Z.one)
for d in 1..=top:
    let (up, down) = (fib[d], fib[d].inv)
    for k in (d..=top).step(d):
        if mu[k / d] == 1: g[k] *= up
        if mu[k / d] == -1: g[k] *= down
let upto = prefix_fold(*, g)
let undo = [x.inv for x in upto]

// the product of g over l..=r
fn span(l: Int, r: Int) -> Z = upto[r + 1] * undo[l]

for a in asks:
    var answer = Z.one
    var l = 1
    while l <= min(a.n, a.m):
        let r = min(a.n / (a.n / l), a.m / (a.m / l))  // the last T with both quotients of l
        answer *= span(l, r) ** ((a.n / l) * (a.m / l))
        l = r + 1
    print(answer)
