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
domain Key = 0..n
let kids: [V]?(l: V, r: V) = read(None: (0, 0))

let tree = Rooted(root: V.first, next: v => kids[v].list)
var gen: [V]Int = 0
for v in tree.Pre.rev:
    if let (l, r) = kids[v]: gen[v] = max(gen[l], gen[r] + 1)
let by_gen = V.group(by: v => gen[v])

var set: [V]PMultiset<Key> = PMultiset.empty()
var key: [V]Key = Key.first!
var next_key = 0
var order: List<V> = []
for g in 0..n:
    for v in by_gen[g]:
        if let (l, r) = kids[v]: set[v] = set[l].with(key[r])
    let ranked = by_gen[g].sorted(by: u => (desc(set[u]), u))
    for run in ranked.runs(by: v => set[v]):
        for v in run: key[v] = Key(next_key)
        next_key += 1
    order.extend(ranked)
print(order)
