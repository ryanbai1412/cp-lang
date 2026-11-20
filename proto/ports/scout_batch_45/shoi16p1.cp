// SHOI 2016 P1, cheapest guards so every marked node is within k of one
// original: corpus/submissions/5445691.cpp
let n: 1..=5e5 = read()
let k: 0..=20 = read()
domain V = 1..=n
let cost: [V]Int = read()
let m: 0..=n = read()
let marked = Set<V>(read::<[0..m]V>())
let es: [0..n - 1](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(es), V.first)

// need(u, d): cheapest way to settle u's subtree where
//   d >= 0: some guard in it reaches d levels above u,
//   d < 0:  the top -d levels of the subtree are covered from outside.
@memo
fn need(u: V, d: -k..=k) -> Int:
    let kids = tree.children(u)
    var best = inf
    if d < k: best = need(u, d + 1)
    if d == k:
        best min= cost[u] + sum(need(v, -k) for v in kids)
    elif d >= 0:
        // one child carries the guard, the rest only need its reach
        if not kids.is_empty:
            best min= sum(need(v, -d) for v in kids) + min(need(v, d + 1) - need(v, -d) for v in kids)
    else:
        best min= sum(need(v, d + 1) for v in kids)
    if d == 0 and u not in marked:
        best min= sum(need(v, 0) for v in kids)
    best

print(need(V.first, 0))
