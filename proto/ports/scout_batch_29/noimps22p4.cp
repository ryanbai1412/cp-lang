// NOI mock 2022 P4: n numbers up to 2000. Each query gives m primes;
// count the subsets of the numbers whose product is divisible by every
// one of them, modulo 998244353.
//
// A number up to 2000 has at most one prime factor of 43 or more (its
// "big" prime); the 13 primes below 43 are "small". Inclusion-exclusion
// runs over the set of small query primes that divide nothing chosen;
// among the numbers that survive, the multiples of each big query prime
// must not all be left out, and everything else is free.
// original: corpus/submissions/4547602.cpp
use std.math.{Sieve}
use std.mod.{Mod}
type Z = Mod<998244353>

const LIMIT = 2000
const SMALL = 13
let sieve = Sieve(LIMIT)
domain Value = 1..=LIMIT
domain Mask = 0..(1 << SMALL)
domain Prime = 0..sieve.primes.len

/// Rank of a prime among the primes up to LIMIT.
var rank: [Value]?Prime = [Value] v => none
for p in Prime: rank[Value(sieve.primes[p.ord])] = p

/// The small primes of `v` as a mask, and its big prime if it has one.
fn split(v: Value) -> (small: Int, big: ?Prime):
    var (small, big): (Int, ?Prime) = (0, none)
    for (p, _) in sieve.factor(v.label):
        let r = rank[Value(p)]!
        if r.ord < SMALL: small |= 1 << r.ord else: big = r
    (small: small, big: big)

let n: Int = read()
var count: [Value]Int = [Value] v => 0
for _ in 0..n:
    let v: Value = read()
    count[v] += 1
let present = [(v: v, parts: split(v)) for v in Value if count[v] > 0]

// alive[s]: numbers with no small prime in s; with[s][p]: those of them
// whose big prime is p.
var alive: [Mask]Int = [Mask] s => 0
var with: [Mask][Prime]Int = [Mask] s => [Prime] p => 0
for s in Mask:
    for x in present:
        if (x.parts.small & s.ord) != 0: continue
        alive[s] += count[x.v]
        if let p = x.parts.big: with[s][p] += count[x.v]

var pow2: List<Z> = [Z(1)]
for _ in 0..n: pow2.push(pow2[pow2.len - 1] * Z(2))

let q: Int = read()
for _ in 0..q:
    let m: Int = read()
    var small = 0
    var big: Set<Prime> = Set()
    for _ in 0..m:
        let p: Value = read()
        let r = rank[p]!
        if r.ord < SMALL: small |= 1 << r.ord else: big.add(r)
    var ans = Z(0)
    var s = small
    while true:                     // every subset s of `small`
        var free = alive[Mask(s)]
        var ways = Z(1)
        for p in big:
            let c = with[Mask(s)][p]
            free -= c
            ways = ways * (pow2[c] - Z(1))
        ways = ways * pow2[free]
        if popcount(s) % 2 == 0: ans += ways else: ans -= ways
        if s == 0: break
        s = (s - 1) & small
    print(ans)
