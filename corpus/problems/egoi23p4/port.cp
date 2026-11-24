// Source: EGOI 2023 Day 1 P4 (Bikes vs Cars), https://dmoj.ca/problem/egoi23p4
// limits: 5000 ms, 1024 MB
// Problem: build a connected network of at most 2023 streets on n <= 500 places; each street has
//   width W <= 1e6, split into a bike lane b and a car lane W - b. For every pair i < j the widest
//   car that can travel between them must be exactly C[i][j] and the widest bike B[i][j]. Print
//   NO, or M and the streets u v b.
// Solution: in bike-lane terms, B is a maximin and W - C a minimax bottleneck matrix. Both must be
//   closed under Floyd-Warshall, and pair (i, j) can only get streets with b in
//   [W - C[i][j], B[i][j]]; when that is non-empty, offer two streets with b at both ends. If the
//   offered streets reproduce both matrices, the minimum and the maximum spanning trees of them
//   by b (Kruskal both ways) keep both bottlenecks with <= 2(n - 1) streets. O(n^3).
// original: corpus/submissions/5678052.cpp
// Bounds: n: 1..=500; kv: 1..=1e9
let n: 1..=500 = read()
let kv: 1..=1e9 = read()
domain V = 0..n

var arr_mx: [V, V]Int = 0
for i in 0..n:
    for j in 0..i:
        let x: Int = read()
        arr_mx[V(i), V(j)] = x
        arr_mx[V(j), V(i)] = x

var arr_mn: [V, V]Int = 0
for i in 0..n:
    for j in 0..i:
        let x: Int = read()
        arr_mn[V(i), V(j)] = x
        arr_mn[V(j), V(i)] = x

for i in V:
    for j in V: arr_mx[i, j] = kv - arr_mx[i, j]
for i in V:
    arr_mn[i, i] = 1_000_000_000
    arr_mx[i, i] = -1

fn matches_mn(src: [V, V]Int, target: [V, V]Int) -> Bool:
    var g: [V, V]Int = 0
    for i in V:
        for j in V: g[i, j] = src[i, j]
    for i in V: g[i, i] = 1_000_000_000
    for mid in V:
        for u in V:
            for v in V: g[u, v] max= min(g[u, mid], g[mid, v])
    for u in V:
        for v in V:
            if g[u, v] != target[u, v]: return false
    true

fn matches_mx(src: [V, V]Int, target: [V, V]Int) -> Bool:
    var g: [V, V]Int = 0
    for i in V:
        for j in V: g[i, j] = src[i, j]
    for i in V: g[i, i] = -1
    for mid in V:
        for u in V:
            for v in V: g[u, v] min= max(g[u, mid], g[mid, v])
    for u in V:
        for v in V:
            if g[u, v] != target[u, v]: return false
    true

fn solve():
    if not matches_mn(arr_mn, arr_mn) or not matches_mx(arr_mx, arr_mx):
        print("NO")
        return

    var edges: List<(Int, V, V)> = []
    var cg_mn: [V, V]Int = -1
    var cg_mx: [V, V]Int = 1_000_000_000
    for i in 0..n:
        for j in 0..i:
            let ui = V(i)
            let vj = V(j)
            let w1 = arr_mn[ui, vj]
            let w2 = arr_mx[ui, vj]
            if w1 >= w2:
                edges.push((w1, ui, vj))
                edges.push((w2, ui, vj))
                cg_mn[ui, vj] max= w1
                cg_mn[vj, ui] max= w1
                cg_mx[ui, vj] min= w2
                cg_mx[vj, ui] min= w2

    if not matches_mn(cg_mn, arr_mn) or not matches_mx(cg_mx, arr_mx):
        print("NO")
        return

    edges.sort()
    var keep: List<(V, V, Int)> = []
    var d1 = Dsu<V>()
    for idx in 0..edges.len:
        let e = edges[idx]
        if d1.unite(e.1, e.2): keep.push((e.1, e.2, e.0))
    var d2 = Dsu<V>()
    for idx in (0..edges.len).rev:
        let e = edges[idx]
        if d2.unite(e.1, e.2): keep.push((e.1, e.2, e.0))

    print(keep.len)
    for idx in 0..keep.len:
        let e = keep[idx]
        print(e.0.idx, e.1.idx, e.2)

solve()
