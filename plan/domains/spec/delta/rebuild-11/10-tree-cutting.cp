// Codeforces 1946C: Tree Cutting
// I/O: t tests; each n k then n-1 edges u v; print the largest x such that removing exactly k edges leaves every component with >= x vertices.
@!multitest
let n: 2..=1e5 = read()
let k: 1..=1e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let t = Rooted(g, root: V.first!)
// greedy bottom-up: cut a subtree off as soon as it reaches x; x works iff more than k pieces form
fn pieces_at_least(x: Int) -> Bool:
    var sz: [V]Int = 1
    var cuts = 0
    for v in t.order.rev:
        for c in t.children(v): sz[v] += sz[c]
        if sz[v] >= x:
            cuts += 1
            sz[v] = 0
    cuts > k
print((1..=n).last_true(pieces_at_least)!)
