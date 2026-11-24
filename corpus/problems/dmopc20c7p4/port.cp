// Source: DMOPC '20 Contest 7 P4, Mimi and Carrots II, https://dmoj.ca/problem/dmopc20c7p4
// Problem: a tree on N <= 2e5 cities, none safe at first, Q <= 2e5 events: "1 u" toggles
//   whether u is safe; "2 u v" prints the number of safe cities on the u-v path or adjacent
//   to a city on it.
// Solution: root the tree and keep near[x], the number of safe children of x, in a SegTree in
//   HLD order. Every counted city is a child of a path vertex except the path's top and its
//   parent, so a query is one path sum plus two lookups; a toggle updates near[parent].
//   O(log^2 N) per query.
// original: corpus/submissions/3686586.cpp

let n: 1..=2e5 = read()
let q: 1..=2e5 = read()
domain City = 1..=n
domain Road = 1..n
let roads: [Road](u: City, v: City) = read()
let tree = Rooted(Graph.undirected(roads), City.first!)

var safe: [City]Bool = false
var safe_children = SegTree<tree.Pre, Int>(+)
for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        let u: City = read()
        safe[u] = not safe[u]
        if let p = tree.parent(u):
            safe_children.update(tree.pos(p), c => c + (if safe[u]: 1 else: -1))
    else:
        let u: City = read()
        let v: City = read()
        let top = tree.lca(u, v)
        let above = if let p = tree.parent(top): Int(safe[p]) else: 0
        print(tree.fold_path(u, v, safe_children) + Int(safe[top]) + above)
