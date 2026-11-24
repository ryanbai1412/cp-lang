// Source: Codeforces 2138C1 (Maple and Tree Beauty, Easy Version),
//   https://codeforces.com/contest/2138/problem/C1
// limits: 3000 ms, 1024 MB
// Problem: a rooted tree of n <= 1000 vertices (parents p_i < i). Label exactly k vertices 0
//   and the rest 1 to maximise the longest common subsequence of all root-to-leaf label strings.
//   Multitest, t <= 50.
// Solution: beauty l is reachable iff depths 0..l-1 are each monochrome (l <= shallowest leaf
//   depth + 1) and some set of those levels plus some deeper vertices has exactly k vertices.
//   Shrinking l only frees vertices, so binary search l. The subset sum is a Bitset over level
//   sizes grouped by size (O(sqrt n) distinct), each group split into powers of two:
//   O(n sqrt(n) log n / 64) per check.

@!multitest
let (n, k): (2..=1000, 0..=1000) = read()
domain V = 1..=n
domain S = 0..=n
let raw_par: [2..=n]V = read()
var par: [V]?V = None
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
