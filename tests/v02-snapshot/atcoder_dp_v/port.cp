// Source: AtCoder Educational DP Contest V (Subtree), https://atcoder.jp/contests/dp/tasks/dp_v
// Problem: a tree of N <= 1e5 vertices. For every vertex v, count the ways to paint vertices
//   black so the black ones are connected and include v, modulo M (M <= 1e9, not prime).
// Solution: rerooting. below[v] = product over children c of (1 + below[c]): sets within v's
//   subtree that contain v. beyond[c] = 1 + (sets on the parent's side containing the parent),
//   built from beyond[v] and prefix/suffix products of the siblings (no division, M may not be
//   prime). The answer at v is below[v] * beyond[v]. O(N).

let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let tree = Rooted(u => g.out(u), V.first!)
let top_down = tree.at

var below: [V]Int = 1 % m
for v in top_down.rev:
    for c in tree.children(v): below[v] = below[v] * (1 + below[c]) % m

var beyond: [V]Int = 1 % m
for v in top_down:
    let kids = tree.children(v).collect()
    // every child sees the parent's side times all of its siblings: keep the
    // product of the siblings before it, and look up the product of those after
    var after = List.filled(kids.len + 1, 1 % m)
    for i in (0..kids.len).rev: after[i] = after[i + 1] * (1 + below[kids[i]]) % m
    var before = beyond[v]
    for i in 0..kids.len:
        beyond[kids[i]] = (1 + before * after[i + 1]) % m
        before = before * (1 + below[kids[i]]) % m

for v in V: print(below[v] * beyond[v] % m)
