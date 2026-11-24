// Source: NOI '22 MPTS P4 (Card), https://dmoj.ca/problem/noimps22p4
// limits: 1000 ms, 512 MB
// Problem: n <= 1e6 cards with values s_i <= 2000, then m <= 1500 queries, each a list of primes
//   <= 2000 (total length <= 18000). For each query count the subsets of cards whose product is
//   divisible by every listed prime, mod 998244353.
// Solution: a value <= 2000 has at most one prime factor above 41, the 13th prime, so a card
//   is a mask of its small primes and at most one big prime. Sum-over-subsets counts, for every
//   mask, the cards whose small primes lie inside it, overall and per big prime. A query does
//   inclusion-exclusion over the submasks S of its small primes: of the cards avoiding S, each
//   queried big prime b needs one of its c_b cards (2^c_b - 1) and the rest are free (2^rest),
//   sign (-1)^|S|. Cost: SOS over 2^13 masks per big prime, then 2^|small| * |big| per query.
// Bounds: n: 1..=1e6; cards: 1..=2000; q: 1..=1500; c: 1..=18000; asked: 2..=2000
use std.math.{submasks}

type Z = Mod<998244353>

let n: 1..=1e6 = read()
domain Val = 1..=2000
let cards: [n]Val = read()

let sieve = Sieve(Val.count)
let primes = [p for p in 2..=Val.count if sieve.is_prime(p)]
let smalls = 13
var rank: [Val]?Int = None  // where a prime sits in `primes`
for (j, p) in primes.items: rank[Val(p)] = Some(j)

var count: [Val]Int = 0
for c in cards: count[c] += 1

// below[m]: cards whose small primes all lie in m; below_big[b][m]: those of them with big prime b
var below = List.filled(1 << smalls, 0)
var below_big = [List.filled(1 << smalls, 0) for _ in smalls..primes.len]
for v in Val if count[v] > 0:
    var mask = 0
    var big: ?Int = None
    for (p, _) in sieve.factor(v.label):
        let j = rank[Val(p)]!
        if j < smalls: mask |= 1 << j
        else: big = Some(j - smalls)
    below[mask] += count[v]
    if let b = big: below_big[b][mask] += count[v]
sos(mut below)
for b in below_big.keys: sos(mut below_big[b])

var pow2: List<Z> = [Z.one]
for _ in 0..n: pow2.push(pow2.last! * 2)

let full = (1 << smalls) - 1
let q: 1..=1500 = read()
for _ in 0..q:
    let c: 1..=18000 = read()
    let asked: [c](2..=2000) = read()
    var need = 0
    var bigs: List<Int> = []
    for p in asked:
        let j = rank[Val(p)]!
        if j < smalls: need |= 1 << j
        else: bigs.push(j - smalls)
    let wanted = bigs.sorted().deduped()
    var ans: Z = 0
    for s in submasks(need):
        let rest = full ^ s
        var free = below[rest]
        var term: Z = 1
        for b in wanted:
            let with_b = below_big[b][rest]
            free -= with_b
            term *= pow2[with_b] - 1
        term *= pow2[free]
        if popcount(s) % 2 == 0: ans += term
        else: ans -= term
    print(ans)
