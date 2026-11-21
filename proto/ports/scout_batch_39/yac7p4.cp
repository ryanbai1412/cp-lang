// YAC 7 P4 — original: corpus/submissions/5705324.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let (n, m): (1..=200_000, 1..=200_000) = read()
domain V = 1..=n
let edges: [0..n - 1](V, V) = read()
let queries: [0..m]V = read()

let g = Graph.undirected(edges)
let t = Rooted<V>(g, V.first)

var active: [V]Bool = true
// FRICTION 39.3: Rooted<V>.sub(v) returns Range<Pre> without specifying the domain spelling (t.Pre vs Pre)
var tins = Set<t.Pre>()
var adj: [V]Set<V> = [V] _ => Set<V>()
var leaves = Set<V>()

for v in V:
    tins.add(t.sub(v).first!)                   // FRICTION 39.3: no t.pre(v); use t.sub(v).first!
for (u, v) in edges:
    adj[u].add(v)
    adj[v].add(u)

fn sync_leaf(u: V):
    if adj[u].len == 1: leaves.add(u)
    else: leaves.remove(u)

for v in V:
    sync_leaf(v)

fn has_active_sub(u: V) -> Bool:
    let sub = t.sub(u)
    if let p = tins.first_ge(sub.first!): p <= sub.last!
    else: false

fn closest_edge(u: V) -> (V, V):
    // FRICTION 39.3: Pre -> V inverse relies on t.preorder[p] indexing
    let top = t.at[tins.first!]
    if not t.is_anc(top, u):
        return (top, t.parent(top)!)
    // FRICTION 39.3: lift_while polarity (highest-true vs first-false) & return type (?V vs V) unspecified
    let below = t.lift_while(u, v => not has_active_sub(v))
    (t.parent(below)!, below)

for u in queries:
    if tins.len == 1:
        print(1)
        continue

    var to_erase = leaves.copy()
    var to_insert: ?(V, V) = none
    if active[u]:
        to_erase.remove(u)
    else:
        let (c1, c2) = closest_edge(u)
        to_erase.remove(c1)
        to_insert = (c1, c2)

    for a in to_erase:
        active[a] = false
        tins.remove(t.sub(a).first!)
        for v in adj[a]:
            adj[v].remove(a)
            sync_leaf(v)
        adj[a].clear()
        leaves.remove(a)

    if let (c1, c2) = to_insert:
        active[c2] = true
        tins.add(t.sub(c2).first!)
        adj[c1].add(c2)
        adj[c2].add(c1)
        sync_leaf(c1)
        sync_leaf(c2)

    print(tins.len)
