use std.tree.{Rooted}
use std.persistent.{PMultiset}

// f_u = x ^ (the product of f over the right children down u's left spine), so a node is
// the multiset of those right children, compared largest first. A value only holds values
// of lower generation (gen = max(gen l, gen r + 1)) and beats every value of a lower one,
// so ranking generation by generation gives each distinct value a key in increasing order.

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
let kids: [V]?(l: V, r: V) = read(absent: (0, 0))

let tree = Rooted(root: V.first, kids: v => kids[v].list)
var gen: [V]Int = 0
var by_gen: [0..n]List<V> = []
for v in tree.post_order:
    if let (l, r) = kids[v]: gen[v] = max(gen[l], gen[r] + 1)
    by_gen[gen[v]].push(v)

// Hashed persistent multisets: equality is O(1) and `cmp(desc:)` walks to the largest key
// where two differ, O(log n).
var set: [V]PMultiset<0..n> = PMultiset.empty
var key: [V]0..n = 0
var keys = 0
var order: List<V> = []
for g in 0..n:
    for v in by_gen[g]:
        if let (l, r) = kids[v]: set[v] = set[l].with(key[r])
    let ranked = sorted(by_gen[g], cmp: (u, v) => set[u].cmp(set[v], desc: true) then u <=> v)
    for (_, run) in runs_by(ranked, v => set[v]):
        for v in run: key[v] = keys
        order.extend(run)
        keys += 1
print(order)
