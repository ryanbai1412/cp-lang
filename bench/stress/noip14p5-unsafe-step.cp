// NOIP 2014 Finding the Way (proto/ports/noip14p5.cp), with a planted bug:
// the walk checks only the vertex it leaves, so it may step onto a vertex
// whose out-neighbours cannot all reach t. The arcs are read as a
// `Digraph`, which stress makes and shrinks as one.
use std.graph.{Digraph}

let n: 1..=10000 = read()
let m: 0..=200000 = read()
domain V = 1..=n
domain E = 0..m
let edges: Digraph<V, E> = read()
let s: V = read()
let t: V = read()

var out: [V]List<V> = []
var back: [V]List<V> = []
for (u, v) in edges:
    out[u].push(v)
    back[v].push(u)

let reaches = bfs(v => back[v], from: [t])
let safe = [V] u => out[u].all(v => reaches[v] != none)

fn way() -> Int:
    let walk: [V]List<V> = [V] u => [v for v in out[u] if safe[u]]
    if safe[s]: bfs(v => walk[v], from: [s])[t] ?? -1 else: -1

@brute(way)
fn way_slow() -> Int:
    let walk: [V]List<V> = [V] u => [v for v in out[u] if safe[u] and safe[v]]
    if safe[s]: bfs(v => walk[v], from: [s])[t] ?? -1 else: -1

print(way())
