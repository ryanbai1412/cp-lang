use std.rand.{Rng}
use std.seq.{run_ranges}
use std.tree.{Rooted}

// f_u = x ^ (the product of f over the right children down u's left spine), so a node is
// the multiset of those right children, compared largest first. A value only holds values
// of lower generation (gen = max(gen l, gen r + 1)) and beats every value of a lower one,
// so ranking generation by generation gives each distinct value a key in increasing order.
// A multiset is a persistent segment tree over keys with random-weighted hashes; two
// differ first at the largest key whose subtrees hash apart.

type Ix = 0..=10_000_000

struct Node(l: Ix, r: Ix, cnt: 0..=400_000, h: U64)

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
let kids: [V](Int, Int) = read()

fn split(v: V) -> ?(l: V, r: V):
    let (l, r) = kids[v]
    if l == 0: none else: some((V(l), V(r)))

let tree = Rooted(root: V.first, adj: v => if let (l, r) = split(v): [l, r] else: List<V>())
var gen: [V]Int = 0
var by_gen: [0..n]List<V> = []
for p in tree.Pre.rev:
    let v = tree.at[p]
    if let (l, r) = split(v): gen[v] = max(gen[l], gen[r] + 1)
    by_gen[gen[v]].push(v)

var rng = Rng.new(2135)
let weight: [0..n]U64 = [0..n] _ => rng.next()
var pool = [Node(0, 0, 0, U64(0))]

fn insert(t: Ix, lo: Int, hi: Int, k: Int) -> Ix:
    let at = pool[t]
    let mid = (lo + hi) / 2
    let (l, r) = if hi - lo == 1: (0, 0) elif k < mid: (insert(at.l, lo, mid, k), at.r) else: (
        at.l,
        insert(at.r, mid, hi, k),
    )
    pool.push(Node(l, r, at.cnt + 1, at.h + weight[k]))
    pool.len - 1

fn less(a0: Ix, b0: Ix) -> Bool:
    var (a, b, lo, hi) = (a0, b0, 0, n)
    while hi - lo > 1 and pool[a].h != pool[b].h:
        let mid = (lo + hi) / 2
        if pool[pool[a].r].h != pool[pool[b].r].h:
            (a, b, lo) = (pool[a].r, pool[b].r, mid)
        else:
            (a, b, hi) = (pool[a].l, pool[b].l, mid)
    pool[a].cnt < pool[b].cnt

struct Ranked(t: Ix, v: V)

impl Ord for Ranked:
    fn lt(a, b) = if pool[a.t].h == pool[b.t].h: a.v < b.v else: less(a.t, b.t)

var set: [V]Ix = 0
var key: [V]Int = 0
var keys = 0
var order: List<V> = []
for g in 0..n:
    for v in by_gen[g]:
        if let (l, r) = split(v): set[v] = insert(set[l], 0, n, key[r])
    let ranked = [Ranked(set[v], v) for v in by_gen[g]].sorted()
    for r in ranked.run_ranges(by: x => pool[x.t].h):
        for i in r:
            key[ranked[i].v] = keys
            order.push(ranked[i].v)
        keys += 1
print(order)
