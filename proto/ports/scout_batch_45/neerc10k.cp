// NEERC 2010 K, K-Graph Oddity: colour a connected odd graph with the least odd k >= max degree
// original: corpus/submissions/5325567.cpp
let n: 1..=9999 = read()
let m: 0..=1e5 = read()
domain V = 1..=n
let es: [0..m](u: V, v: V) = read()
let g = Graph.undirected(es)

let k = max(g[v].len for v in V) | 1
domain Colour = 1..=k

// Start a BFS at a vertex with a spare colour (degree < k; n odd guarantees
// one), then colour in reverse: everyone but the start still has an
// uncoloured BFS parent, so at most k - 1 colours are taken around it.
let start = V.first(v => g[v].len < k)!
var colour: [V]?Colour = none
for u in bfs(g, from: start).order.rev:
    let taken = Set(colour[v] for v in g[u])
    colour[u] = Colour.first(c => c not in taken)

print(k)
for v in V: print(colour[v]!)

@checker
fn check():
    let got_k: Int = read_output()
    let got: [V]Colour = read_output()
    assert got_k == k
    assert all(got[e.u] != got[e.v] for e in es)
