// Source: NOI '22 MPTS P4 (Card), https://dmoj.ca/problem/noimps22p4
// Problem: n <= 1e6 cards with values s_i <= 2000, then m <= 1500 queries, each a list of primes
//   <= 2000 (total length <= 18000). For each query count the subsets of cards whose product is
//   divisible by every listed prime, mod 998244353.
// Solution: a value <= 2000 has at most one prime factor above 41, so split primes into the 13
//   small ones (a bitmask) and the big one. Subset sums (SOS) over small masks count cards whose
//   small primes lie inside a mask, overall and per big prime. A query does inclusion-exclusion
//   over subsets S of its small primes: of the cards avoiding S, each queried big prime b needs
//   one of its c_b cards (2^c_b - 1) and the rest are free (2^rest), sign (-1)^|S|. Cost: SOS
//   over 2^13 masks per big prime, then 2^|small| * |big| per query.

type Z = Mod<998244353>

let max_v = 2000
let small_cnt = 13
let full = (1 << small_cnt) - 1

let n: Int = read()
var cnt = List.filled(max_v + 1, 0)
for _ in 0..n:
    let v: Int = read()
    cnt[v] += 1

let sieve = Sieve(max_v)
var small_p: List<Int> = []
var big_id = List.filled(max_v + 1, 0)
var big_cnt = 0
for p in 2..=max_v:
    if sieve.is_prime(p):
        if small_p.len < small_cnt:
            small_p.push(p)
        else:
            big_cnt += 1
            big_id[p] = big_cnt

var small_mask = List.filled(max_v + 1, 0)
var val_big = List.filled(max_v + 1, 0)
for v in 1..=max_v:
    var x = v
    var sm = 0
    for j in 0..small_cnt:
        let p = small_p[j]
        if x % p == 0:
            sm = sm | (1 << j)
            while x % p == 0: x /= p
    small_mask[v] = sm
    if x > 1:
        val_big[v] = big_id[x]

var total_sub = List.filled(1 << small_cnt, 0)
// Store only non-zero (big_id, mask, count) buckets per big prime, then SOS on demand
// or precompute SOS over small_mask for total and per big prime that appears in input!
var active_big = List.filled(big_cnt + 1, false)
var big_vals: List<List<(Int, Int)>> = [[] for _ in 0..=big_cnt]

for v in 1..=max_v:
    if cnt[v] > 0:
        let sm = small_mask[v]
        total_sub[sm] += cnt[v]
        let b = val_big[v]
        if b > 0:
            active_big[b] = true
            big_vals[b].push((sm, cnt[v]))

for bit in 0..small_cnt:
    let step = 1 << bit
    for mask in 0..(1 << small_cnt):
        if (mask & step) != 0:
            total_sub[mask] += total_sub[mask ^ step]

var big_sub: Map<Int, List<Int>> = Map()
for b in 1..=big_cnt:
    if active_big[b]:
        var arr = List.filled(1 << small_cnt, 0)
        let vs = big_vals[b]
        for idx in 0..vs.len:
            let (sm, c) = vs[idx]
            arr[sm] += c
        for bit in 0..small_cnt:
            let step = 1 << bit
            for mask in 0..(1 << small_cnt):
                if (mask & step) != 0:
                    arr[mask] += arr[mask ^ step]
        big_sub[b] = arr

var pow2: List<Z> = List.filled(n + 1, Z.one)
for i in 1..=n:
    pow2[i] = pow2[i - 1] + pow2[i - 1]

fn popcount(x0: Int) -> Int:
    var x = x0
    var r = 0
    while x > 0:
        r += x & 1
        x = x >> 1
    r

let q: Int = read()
for _ in 0..q:
    let m: Int = read()
    var sm_req = 0
    var lg_seen: Map<Int, Bool> = Map()
    var lg_req: List<Int> = []
    for _ in 0..m:
        let p: Int = read()
        if big_id[p] > 0:
            let b = big_id[p]
            if not (lg_seen.get(b) ?? false):
                lg_seen[b] = true
                lg_req.push(b)
        else:
            for j in 0..small_cnt:
                if small_p[j] == p:
                    sm_req = sm_req | (1 << j)

    var ans: Z = 0
    var sub = sm_req
    while true:
        let comp = full ^ sub
        var free_cnt = total_sub[comp]
        var prod: Z = 1
        for idx in 0..lg_req.len:
            let b = lg_req[idx]
            var cb = 0
            if let arr = big_sub.get(b):
                cb = arr[comp]
            free_cnt -= cb
            prod *= pow2[cb] - Z.one
        let term = pow2[free_cnt] * prod
        if popcount(sub) % 2 == 0:
            ans += term
        else:
            ans -= term
        if sub == 0: break
        sub = (sub - 1) & sm_req

    print(ans)
