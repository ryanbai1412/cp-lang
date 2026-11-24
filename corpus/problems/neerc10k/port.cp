// Source: NEERC 2010 K (K-Graph Oddity), https://dmoj.ca/problem/neerc10k
// Problem: a connected graph, odd n <= 9999 vertices, m <= 1e5 edges. Print the least odd
//   k >= the maximum degree and a proper colouring of the vertices with colours 1..=k.
// Solution: n and k odd means not every degree is k, so start a BFS at a vertex of degree
//   < k. Colour greedily in reverse BFS order: every other vertex still has an uncoloured BFS
//   parent, so at most k - 1 colours are taken around it. O(m + nk).
// original: corpus/submissions/5325567.cpp
// Bounds: n: 1..=9999; m: 0..=1e5; u, v: 1..=n
let n: 1..=9999 = read()
let m: 0..=1e5 = read()
domain V = 1..=n
var adj: [V]List<V> = []
for _ in 0..m:
    let u: V = read()
    let v: V = read()
    adj[u].push(v)
    adj[v].push(u)

var k = 0
for v in V: k max= adj[v].len
if k % 2 == 0: k += 1

// Start a BFS at a vertex with a spare colour (degree < k; n odd guarantees
// one), then colour in reverse: everyone but the start still has an
// uncoloured BFS parent, so at most k - 1 colours are taken around it.
var start = V.first!
for v in V:
    if adj[v].len < k: start = v
var order: List<V> = []
var seen: [V]Bool = false
order.push(start)
seen[start] = true
var head = 0
while head < order.len:
    let u = order[head]
    head += 1
    for v in adj[u]:
        if not seen[v]:
            seen[v] = true
            order.push(v)

var colour: [V]Int = 0
for at in (0..order.len).rev:
    let u = order[at]
    var taken: List<Bool> = [false for _ in 0..=k]
    for v in adj[u]: taken[colour[v]] = true
    var c = 1
    while taken[c]: c += 1
    colour[u] = c
print(k)
for v in V: print(colour[v])
