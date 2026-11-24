// Source: DMOPC '20 Contest 7 P4, Mimi and Carrots II, https://dmoj.ca/problem/dmopc20c7p4
// Problem: a tree on N <= 2e5 cities, none safe at first, Q <= 2e5 events: "1 u" toggles
//   whether u is safe; "2 u v" prints the number of safe cities on the u-v path or adjacent
//   to a city on it.
// Solution: root the tree and keep near[x], the number of safe children of x, in a SegTree in
//   HLD order. Every counted city is a child of a path vertex except the path's top and its
//   parent, so a query is one path sum plus two lookups; a toggle updates near[parent].
//   O(log^2 N) per query.
// original: corpus/submissions/3686586.cpp

let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let t = Rooted(u => g.out(u), V.first!)

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
