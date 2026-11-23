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

@tags("first", "second")
enum Run = First | Second

let sieve = Sieve(V)
let primes = sieve.primes[..P]
// By count of prime factors, x grown only by primes up to its smallest: the C++ order.
fn grow(x: Int) -> List<Int> = primes.take_while(
    p => p <= sieve.spf[x] and x * p <= V
).map(p => x * p)
let symbols = [1].layers(grow).flatten().take(1 << SYMBOL)
let symbol = symbols.positions()
let powers = primes.map(p => p ** V.ilog(p))

let run: Run = read()
for _ in 0..read<Int>():
    match run:
        First:
            let a: List<0..=V> = read()
            let b = a.bits(WORD).chunks(SYMBOL).map(s => symbols[s]) + powers
            print(b.len)
            print(b)
        Second:
            let (n, k): (Int, Int) = read()
            fn ask(i: Int, j: Int) -> Int = query("?", i + 1, j + 1)
            let b = (0..k - P).map(i => symbol[(0..P).map(j => ask(i, k - P + j)).prod()]!)
            print("!", b.bits(SYMBOL).chunks(WORD).take(n))
