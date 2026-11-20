// DMOPC '20 Contest 6 P3 — Directed-Graph Retrograde Game Theory (CP Lang v2 spec)
// original: corpus/submissions/3917634.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Two-player game on a directed graph: each vertex belongs to the OR-player
// (`want[u] == 1`, wins if any outgoing move wins) or the AND-player
// (`want[u] == 0`, wins only if all outgoing moves win); sinks win immediately,
// and unresolved cycle states default to 0.

let (n, m): (1..=5e5, 0..=5e5) = read()
domain V = 1..=n
domain E = 0..m

// FRICTION UD9: Reading `0`/`1` integers directly into `[V]Bool` is undefined
// on the v2 cheat sheet (`@tags` is for single-char tokens, not `0`/`1` ints),
// so we read `[V](0..=1)` and compare `== 1`.
let want_raw: [V](0..=1) = read()
let want: [V]Bool = [V] u => want_raw[u] == 1
let edges: [E](u: V, v: V) = read()

let g = Graph.directed(edges)
// FRICTION UD10: The v2 cheat sheet lists `Graph.directed(es)` and `g[u]`, but
// omits `.transpose()` / `.rev` and `.deg(u)` (even though `stdlib/graph.cp`
// has both), forcing a manual reverse-edge list unless documented on the sheet.
let rev_edges: [E](u: V, v: V) = [E] e => (u: edges[e].v, v: edges[e].u)
let rg = Graph.directed(rev_edges)

var out_deg: [V]Int = [V] u => count(v for v in g[u])
var rem = out_deg.copy()
var queued: [V]Bool = false
var state: [V]?Bool = none
var q: List<V> = []

fn enqueue(u: V, force: Bool):
    if not queued[u] and (force or rem[u] == 0):
        queued[u] = true
        q.push(u)

for u in V:
    enqueue(u, false)

while let u = q.pop():
    let res = if out_deg[u] == 0:
        true
    elif want[u]:
        any(state[v] == true for v in g[u])
    else:
        all(state[v] == true for v in g[u])
    state[u] = res
    for p in rg[u]:
        rem[p] -= 1
        enqueue(p, want[p] and res)

print(" ".join(if state[u] ?? false: "1" else: "0" for u in V))
