// DMOPC '20 Contest 7 P4 - Mimi and Carrots II (17pt): a tree of n burrows,
// each safe or not (all unsafe at first). "1 u" flips burrow u; "2 u v" asks
// how many safe burrows lie on the path from u to v or next to it.
// Keep near[x] = the number of safe children of x. Every safe vertex on or
// beside the path is a child of a path vertex, except the top of the path
// and the parent of the top, so the answer is one path sum plus two lookups.
// original: corpus/submissions/3686586.cpp

let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let t = Rooted(u => g.out(u), V.first)

var safe: [V]Bool = false
var near = SegTree<t.Pre, Int>(+)

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        let u: V = read()
        safe[u] = not safe[u]
        if let p = t.parent(u): near.set(t.pos(p), near.get(t.pos(p)) + (if safe[u]: 1 else: -1))
    else:
        let (u, v): (V, V) = read()
        let top = t.lca(u, v)
        var ans = t.fold_path(u, v, near)
        if safe[top]: ans += 1
        if let p = t.parent(top):
            if safe[p]: ans += 1
        print(ans)
