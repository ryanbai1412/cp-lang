// Codeforces 2135 F: To the Infinity.
// `f_u = x ^ (product of f over right children down u's left spine)`, so each
// vertex represents the multiset of its left-spine right children, compared
// largest element first. Defining generation `gen[u] = max(gen[l], gen[r] + 1)`
// (0 at leaves), every vertex in generation g only contains keys from strictly
// lower generations and outranks every vertex of lower generation. Processing
// generations in order with hashed persistent multisets gives O(1) equality
// and O(log n) comparison, assigning dense ranks generation by generation.

use std.tree.Rooted
use std.persistent.PMultiset

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
domain Gen = 0..n
let kids: [V]?(l: V, r: V) = read(none: (0, 0))

let tree = Rooted(root: V.first, adj: v => kids[v].list)
var gen: [V]Gen = Gen.first
// Keep each generation in bottom-up (`tree.Pre.rev`) order so `l` precedes `v` when `gen[l] == gen[v]`.
var by_gen: [Gen]List<V> = []
for p in tree.Pre.rev:
    let v = tree.at[p]
    if let (l, r) = kids[v]: gen[v] = max(gen[l], (gen[r] + 1)!)
    by_gen[gen[v]].push(v)

var set: [V]PMultiset<0..n> = PMultiset.empty
var key: [V]0..n = 0
var next_key = 0
var order: List<V> = []
for g in Gen:
    for v in by_gen[g]:
        if let (l, r) = kids[v]: set[v] = set[l].with(key[r])
    let ranked = by_gen[g].sorted(cmp: (u, v) => set[u].cmp(set[v], desc: true) then u <=> v)
    let groups = ranked.runs(by: v => set[v])
    for (r, (_, run)) in groups.items:
        for v in run: key[v] = next_key + r
    next_key += groups.len
    order.extend(ranked)
print(order)
