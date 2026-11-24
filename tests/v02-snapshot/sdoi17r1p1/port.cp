// Source: SDOI 2017 Round 1, Number Table (Luogu P3704), https://dmoj.ca/problem/sdoi17r1p1
// Problem: multitest (1 <= T <= 1000): given 1 <= n, m <= 1e6, print the product of f(gcd(i, j)) over
//   1 <= i <= n, 1 <= j <= m, where f is Fibonacci (f0 = 0, f1 = 1), modulo 1e9+7.
// Solution: Group by d = gcd and apply Mobius inversion: with g(T) = product over d | T of
//   f(d)^mu(T/d), the answer is the product over T of g(T)^((n/T)(m/T)). Sieve mu, build g by
//   a harmonic loop in O(N log N), keep prefix products of g and their inverses; per query
//   n/T and m/T are constant on O(sqrt n) blocks, one power each: O(sqrt(n) log P).
// original: corpus/submissions/5463988.cpp

const P = 1000000007
type Z = Mod<P>

let t: 1..=1000 = read()
domain Ask = 0..t
let asks: [Ask](n: 1..=1e6, m: 1..=1e6) = read()
let top = [max(a.n, a.m) for a in asks].max()!

// Mobius function from smallest prime factors
let sieve = Sieve(top)
var mu = List.filled(top + 1, 1)
for x in 2..=top:
    let p = sieve.factor(x)[0].0
    mu[x] = if x / p % p == 0: 0 else: -mu[x / p]

var fib = List.filled(top + 2, Z.one)
fib[0] = 0
for i in 2..=top: fib[i] = fib[i - 1] + fib[i - 2]

// g[T], then its prefix products and their inverses
var g = List.filled(top + 1, Z.one)
for d in 1..=top:
    let back = fib[d].inv
    var mult = d
    while mult <= top:
        if mu[mult / d] == 1: g[mult] *= fib[d]
        if mu[mult / d] == -1: g[mult] *= back
        mult += d
var upto = List.filled(top + 1, Z.one)
for i in 1..=top: upto[i] = upto[i - 1] * g[i]
let undo = [x.inv for x in upto]

for a in asks:
    var (answer, l) = (Z.one, 1)
    while l <= min(a.n, a.m):
        let r = min(a.n / (a.n / l), a.m / (a.m / l))  // last T with the same two quotients
        answer *= (upto[r] * undo[l - 1]) ** ((a.n / l) * (a.m / l))
        l = r + 1
    print(answer)
