// Codeforces 2237 G: two runs. The first sees a and writes b; the second sees
// only gcd(b_i, b_j) for pairs it asks about, and must rebuild a.
//
// Every a_i <= 1e6 < 2^20, so a is a stream of 20-bit words, recut into 18-bit
// symbols. Symbol s is written as the s-th number at most V whose primes are all
// among the first P = 150; more than 2^18 such numbers exist. After the symbols,
// b holds q_p, the largest power of each of those primes that is at most V. For
// such an x <= V, gcd(x, q_p) is exactly the p-part of x, so the product of the
// P gcds is x itself, which the second run looks up.
@!interactive
const V = 1_000_000
const P = 150
const WORD = 20
const SYMBOL = 18

// Bits a[0] low to high, then a[1], ..., regrouped into `len` words of `to` bits.
fn recut(a: List<Int>, from: Int, to: Int, len: Int) -> List<Int>:
    let bits = [x.bit(b) for x in a for b in 0..from]
    [sum(Int(bits[i * to + b]) << b for b in 0..to if i * to + b < bits.len) for i in 0..len]

let sieve = Sieve(V)
let primes = sieve.primes[..P]

// Smooth numbers by count of prime factors; x grows only by primes up to its
// smallest, so each is listed once (the C++ order, which the output depends on).
// Tuned: the smooth numbers grow in one list read from the front, as
// orig.cpp does, instead of a new list per layer and per number, and the
// symbol table is 32-bit.
var smooth = [1]
var at = 0
while smooth.len < 1 << SYMBOL:
    let x = smooth[at]
    let cap = if x == 1: primes.last! else: sieve.spf[x]
    for p in primes:
        if p > cap or x * p > V: break
        smooth.push(x * p)
    at += 1
let symbols = smooth[..1 << SYMBOL]
var symbol: [0..=V]?(0..1 << SYMBOL) = None
for (s, x) in symbols.items: symbol[x] = s

fn top_power(p: Int) -> Int:
    var q = p
    while q <= V / p: q *= p
    q
let powers = primes.map(top_power)

// The product of gcd(b_i, q_p) over all p, or none once the judge answers -1.
fn decode(i: Int, k: Int) -> ?Int:
    var x = 1
    for j in 0..P:
        print("?", i + 1, k + j + 1)
        let g: Int = read()
        if g == -1: return None
        x *= g
    symbol[x]

let first_run = read<Str>() == "first"
let t: Int = read()
for _ in 0..t:
    if first_run:
        let n: Int = read()
        let a: [0..n]Int = read()
        let b = [
            symbols[s] for s in recut([x for x in a], WORD, SYMBOL, ceil_div(n * WORD, SYMBOL))
        ] + powers
        print(b.len)
        print(b)
    else:
        let (n, k_all): (Int, Int) = read()
        let k = k_all - P
        var b = List<Int>()
        for i in 0..k:
            if let s = decode(i, k): b.push(s) else: return
        print("!", recut(b, SYMBOL, WORD, n))
