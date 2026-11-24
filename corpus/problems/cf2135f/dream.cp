// Codeforces 2135 F: To the Infinity.
// `f_u = x ^ (product of f over right children down u's left spine)`, so each
// vertex represents the multiset of its left-spine right children, compared
// largest element first. Defining generation `gen[u] = max(gen[l], gen[r] + 1)`
// (0 at leaves), every vertex in generation g only contains keys from strictly
// lower generations and outranks every vertex of lower generation. Processing
// generations in order with hashed persistent multisets gives O(1) equality
// and O(log n) comparison, assigning dense ranks generation by generation.

use std.persistent.{PArena, PMultiset}

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
domain Key = 0..n
let kids: [V]?(l: V, r: V) = read<[V](0..=n, 0..=n)>().map(
    (l, r) => if l == 0: None else: Some((V(l), V(r)))
)

fn children(v: V) -> List<V> = if let (l, r) = kids[v]: [l, r] else: []
let tree = Rooted(root: V.first!, next: children)
var gen: [V]Int = 0
for v in tree.order.rev:
    if let (l, r) = kids[v]: gen[v] = max(gen[l], gen[r] + 1)
var by_gen: [0..n]List<V> = []
for v in tree.order.rev: by_gen[gen[v]].push(v)

var ar = PArena<PMultiset<Key>>()
var set: [V]PMultiset<Key> = PMultiset<Key>.empty()
var key: [V]Key = Key.first!
var next_key = 0
var order: List<V> = []
for g in 0..n:
    for v in by_gen[g]:
        if let (l, r) = kids[v]: set[v] = set[l].with(mut ar, key[r])
    let ranked = by_gen[g].sorted(cmp: (a, b) =>
        let c = set[a].cmp(ar, set[b], desc: true)
        if c == 0: a < b else: c < 0
    )
    for run in ranked.runs(by: v => set[v]):
        for v in run: key[v] = Key(next_key)
        next_key += 1
    order.extend(ranked)
print(order)
