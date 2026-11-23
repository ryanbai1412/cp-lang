// Source: Codeforces 2237G, Send GCDs, https://codeforces.com/contest/2237/problem/G
// Problem: run-twice interactive, multitest, sum n <= 1000. Run 1 gets a (a_i <= 1e6) and
//   prints b, k <= ceil(10n/9) + 150 values <= 1e6. Run 2 gets only n and k and may ask
//   gcd(b_i, b_j) at most 180n + 150 times; it must print a.
// Solution: a is a stream of 20-bit words, recut into ceil(20n/18) 18-bit symbols. Symbol s is
//   sent as the s-th 150-smooth number <= 1e6 (there are over 2^18, listed once each by a
//   sieve walk); the last 150 entries of b are q_p, the largest power <= 1e6 of each of the
//   first 150 primes. A smooth x is the product of gcd(x, q_p), so 150 queries decode a symbol
//   by table lookup: 150 ceil(10n/9) <= 180n + 150 queries; recut the symbols to 20-bit words.
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
fn grow(x: Int) -> List<Int>:
    let cap = if x == 1: primes.last! else: sieve.spf[x]
    var out = List<Int>()
    for p in primes:
        if p > cap or x * p > V: break
        out.push(x * p)
    out

var smooth = List<Int>()
var layer = [1]
while smooth.len < 1 << SYMBOL:
    smooth += layer
    layer = [y for x in layer for y in grow(x)]
let symbols = smooth[..1 << SYMBOL]
var symbol: [0..=V]?Int = None
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
