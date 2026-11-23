// SDOI 2017 Round 1 P1, Number Table: the product of fib(gcd(i, j)) over
// 1 <= i <= n, 1 <= j <= m, modulo 1e9+7, for up to 1000 pairs (n, m).
// Group by d = gcd: with g(T) = product over d | T of fib(d) ^ mu(T / d),
// the answer is the product over T of g(T) ^ ((n / T) * (m / T)), and
// n / T and m / T are constant on O(sqrt) blocks of T.
// original: corpus/submissions/5463988.cpp

const P = 1000000007
type Z = Mod<P>

let t: Int = read()
domain Ask = 0..t
let asks: [Ask](n: Int, m: Int) = read()  // n, m <= 1e6
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
