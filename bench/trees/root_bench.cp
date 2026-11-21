// Matched benchmark for building a Rooted from an edge list plus one bottom-up subtree pass.
let n: 1..=1_000_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted.new(Graph.undirected(edges), V.first)
var s: [V]Int = [V] v => v.to_idx
for v in t.at.rev:
    if let p = t.parent(v): s[p] += s[v]
var hash = 0
for v in V: hash = (hash * 31 + s[v] % 1_000_003 * (t.pos(v).ord + 1) + t.depth(v)) % 1_000_000_007
print(hash)
