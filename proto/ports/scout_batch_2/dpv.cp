// AtCoder DP Contest V Subtree: for every vertex v of a tree, count the
// ways to paint a connected set of vertices black that contains v, modulo m.
//
// Rerooting. below[v] counts the sets that stay in v's subtree. beyond[v]
// is one more than the count for the rest of the tree as seen from v's
// parent (the "one more" is leaving that side white). The answer at v is
// their product.

let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let tree = Rooted(u => g.out(u), V.first)
let top_down = tree.at

var below: [V]Int = 1 % m
for v in top_down.rev:
    for c in tree.children(v): below[v] = below[v] * (1 + below[c]) % m

var beyond: [V]Int = 1 % m
for v in top_down:
    let kids = tree.children(v)
    // every child sees the parent's side times all of its siblings: keep the
    // product of the siblings before it, and look up the product of those after
    var after = List.filled(kids.len + 1, 1 % m)
    for i in (0..kids.len).rev: after[i] = after[i + 1] * (1 + below[kids[i]]) % m
    var before = beyond[v]
    for i in 0..kids.len:
        beyond[kids[i]] = (1 + before * after[i + 1]) % m
        before = before * (1 + below[kids[i]]) % m

for v in V: print(below[v] * beyond[v] % m)
