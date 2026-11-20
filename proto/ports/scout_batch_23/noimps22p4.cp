// NOI '22 MPTS P4 - Card (DMOJ: noimps22p4, 25pt, TL 1.0s, ML 512MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// N cards with values s_i <= 2000. M queries each specify a set Q of primes <= 2000.
// Count subsets of cards whose product is divisible by every prime in Q, mod 998244353.
//
// Split primes into the 14 small primes <= 43 (bitmask 0..2^14) and large primes > 43.
// Every card <= 2000 has at most one large prime factor.
// Precompute SOS (zeta transform) over the 14 small-prime bits for:
//   - tot_sub[A]: number of all cards whose small-prime mask is a subset of A
//   - big_sub[p][A]: number of cards divisible by large prime p with small mask <= A
// For a query Q = S (small mask) U L (large primes), by inclusion-exclusion on T <= S:
//   ans = sum_{T <= S} (-1)^|T| * 2^(tot_sub[~T] - sum_{p in L} big_sub[p][~T])
//                               * prod_{p in L} (2^(big_sub[p][~T]) - 1).
use std.mod.{Mod}
use std.math.{Sieve}

type Z = Mod<998244353>
const MAX_V = 2000
const SMALL_CNT = 14
const FULL = (1 << SMALL_CNT) - 1

let n: 1..=1e6 = read()
domain Card = 0..n
let cards: [Card](1..=MAX_V) = read()

let sieve = Sieve(MAX_V)
let small_primes = [p for p in sieve.primes if p <= 43]
let big_primes = [p for p in sieve.primes if p > 43]

var small_idx = List.filled(MAX_V + 1, -1)
for i in 0..small_primes.len: small_idx[small_primes[i]] = i
var big_idx = List.filled(MAX_V + 1, -1)
for i in 0..big_primes.len: big_idx[big_primes[i]] = i

var freq = List.filled(MAX_V + 1, 0)
for i in Card: freq[cards[i]] += 1

domain Mask = 0..(1 << SMALL_CNT)
domain Big = 0..big_primes.len
var tot_sub: [Mask]Int = 0
var big_sub: [Big, Mask]Int = 0

for v in 1..=MAX_V:
    if freq[v] == 0: continue
    var mask = 0
    var big = -1
    for (p, _) in sieve.factor(v):
        if small_idx[p] >= 0: mask |= 1 << small_idx[p]
        else: big = big_idx[p]
    tot_sub[Mask(mask)] += freq[v]
    if big >= 0: big_sub[Big(big), Mask(mask)] += freq[v]

// WISH MG7: SOS / Zeta-Mobius subset transform is missing from std.poly / std.bits
fn sos(mut a: [Mask]Int):
    for b in 0..SMALL_CNT:
        let step = 1 << b
        for m in 0..(1 << SMALL_CNT):
            if (m & step) != 0:
                a[Mask(m)] += a[Mask(m ^ step)]

sos(mut tot_sub)
for b in Big:
    // WISH MG13: passing a lent 2D row `mut big_sub[b]` into `sos(mut a: [Mask]Int)`
    sos(mut big_sub[b])

var pow2 = List.filled(n + 1, Z(1))
for i in 1..=n: pow2[i] = pow2[i - 1] * Z(2)

let q_cnt: 1..=1500 = read()
for _ in 0..q_cnt:
    let c: 1..=MAX_V = read()
    // WISH MG11: input elements are constrained to be prime numbers <= 2000
    var sm = 0
    var seen_big: Set<Big> = {}
    var lg: List<Big> = []
    for _ in 0..c:
        let p: 2..=MAX_V = read()
        if small_idx[p] >= 0:
            sm |= 1 << small_idx[p]
        elif big_idx[p] >= 0:
            let b = Big(big_idx[p])
            if not seen_big.contains(b):
                seen_big.insert(b)
                lg.push(b)

    var ans = Z(0)
    // WISH MG8: submask iteration `for sub in submasks(sm)` instead of manual `while`
    var sub = sm
    while true:
        let allow = Mask(FULL ^ sub)
        var free_cnt = tot_sub[allow]
        var prod = Z(1)
        for b in lg:
            let cnt = big_sub[b, allow]
            free_cnt -= cnt
            prod *= pow2[cnt] - Z(1)
        let term = pow2[free_cnt] * prod
        // WISH MG4: integer/modular promotion & sign parity `(-1)^popcount(sub)`
        if popcount(sub) % 2 == 0: ans += term
        else: ans -= term
        if sub == 0: break
        sub = (sub - 1) & sm
    print(ans)
