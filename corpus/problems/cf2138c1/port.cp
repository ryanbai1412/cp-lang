// Codeforces 2138 C1 — Maple and Tree Beauty (Easy Version)
//
// Label `n` tree vertices with `k` zeros and `n - k` ones to maximize the
// longest common subsequence of all root-to-leaf binary strings.
//
// Any common subsequence of length `l` can be achieved by making every level
// `d in 0..l` monochromatic (all 0 or all 1) up to the shallowest leaf depth
// `leaf_depth` (so `l <= leaf_depth + 1`). Level `d` contributes a block of
// `freq[d]` identical bits, while all vertices at depths `>= l` are free unit
// items (weight 1 each). Thus length `l` is achievable iff some subset of
// `{freq[0], ..., freq[l - 1]}` plus at most `sum_{d >= l} freq[d]` unit items
// sums to `k`.
//
// Monotonicity: decreasing `l` by 1 splits the block `freq[l - 1]` into
// `freq[l - 1]` independent unit items, which can only enlarge the set of
// achievable subset sums; hence `is_good(l)` is monotone non-increasing in `l`
// and `bsearch_last` finds the exact maximum.
//
// Cost: grouping equal weights `ff[w]` has `sum_w w * ff[w] = n`, so at most
// `O(sqrt(n))` distinct weights exist; binary splitting each count `ff[w]` into
// power-of-2 bitset shifts `dp |= dp.shifted(take * w)` runs in
// `O((n * sqrt(n) * log n) / 64)` per `is_good` call.

@!multitest
let (n, k): (2..=1000, 0..=1000) = read()
domain V = 1..=n
domain S = 0..=n
let raw_par: [2..=n]V = read()
var par: [V]?V = none
for i in 2..=n: par[V(i)] = raw_par[i]
let t = Rooted.from_parents(par)

let leaf_depth = V.filter(v => t.size(v) == 1).map(v => t.depth(v)).min()!
var freq: [0..n]Int = 0
for v in V: freq[t.depth(v)] += 1

fn is_good(l: Int) -> Bool:
    var ff: [0..=n]Int = 0
    for i in 0..l: ff[freq[i]] += 1
    for i in l..n: ff[1] += freq[i]
    var dp = Bitset<S>.empty().set(S(0), true)
    for w in 1..=n if ff[w] > 0:
        var rem = ff[w]
        var p = 1
        while rem > 0:
            let take = min(p, rem)
            dp |= dp.shifted(take * w)
            rem -= take
            p *= 2
    S(k) in dp

print((0..leaf_depth + 2).last_true(is_good)!)
