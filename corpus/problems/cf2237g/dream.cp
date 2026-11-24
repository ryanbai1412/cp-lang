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

enum Run = First | Second

// Bits a[0] low to high, then a[1], ..., regrouped into `len` words of `to` bits.
fn recut(a: List<Int>, from: Int, to: Int, len: Int) -> List<Int>:
    let bits = [x.bit(b) for x in a for b in 0..from]
    [sum(Int(bits[i * to + b]) << b for b in 0..to if i * to + b < bits.len) for i in 0..len]

let sieve = Sieve(V)
let primes = sieve.primes[..P]
// By count of prime factors, x grown only by primes up to its smallest: the C++ order.
fn grow(x: Int) -> List<Int>:
    let cap = if x == 1: primes.last! else: sieve.spf[x]
    primes.take_while(p => p <= cap and x * p <= V).map(p => x * p).collect()
var smooth = [1]
var layer = [1]
while smooth.len < 1 << SYMBOL:
    layer = [y for x in layer for y in grow(x)]
    smooth += layer
let symbols = smooth[..1 << SYMBOL]
var symbol: [0..=V]?Int = None
for (s, x) in symbols.items: symbol[x] = s
fn top_power(p: Int) -> Int:
    var q = p
    while q <= V / p: q *= p
    q
let powers = primes.map(top_power)

let run = if read<"first" | "second">() == "first": First else: Second
for _ in 0..read<Int>():
    match run:
        First:
            let a: List<0..=V> = read()
            let b = recut(a, WORD, SYMBOL, ceil_div(a.len * WORD, SYMBOL)).map(s => symbols[s]) + powers
            print(b.len)
            print(b)
        Second:
            let (n, k): (Int, Int) = read()
            fn ask(i: Int, j: Int) -> Int:
                print("?", i + 1, j + 1)
                read()
            let b = (0..k - P).map(i => symbol[(0..P).map(j => ask(i, k - P + j)).prod()]!)
            print("!", recut(b, SYMBOL, WORD, n))
