// Source: NOI 2005 P4 (CongCong and KoKo), https://dmoj.ca/problem/noi05p4
// limits: 600 ms, 256 MB
// Problem: an undirected graph, N, E <= 1000, cat at C and mouse at M connected. Each turn the cat
//   takes up to two steps, each to the neighbour nearest the mouse (smallest label on ties), then
//   the mouse stays or moves to a neighbour, all choices equally likely. Print the expected
//   number of turns until they meet, to 3 decimals.
// Solution: the cat's moves are deterministic and the distance drops every turn, so the states
//   (cat, mouse) form a DAG. BFS from every vertex gives all distances and step[cat, mouse]; a
//   memoised E(cat, mouse) = 1 + the average of E over the mouse's deg + 1 choices after the
//   cat's two steps, or 1 if a step catches it. O(n (n + e)).
// original: corpus/submissions/5032733.cpp
let (n, e) = read<(1..=1_000, 1..=1_000)>()
domain V = 1..=n
let (cat, mouse) = read<(V, V)>()
let g = Graph.undirected(read<[e](u: V, v: V)>())
let dist = [V] t => bfs(g, from: [t]).dist
fn toward(c: V, m: V) -> V = if c == m: m else: g.next(c).min(by: v => (dist[m][v]!, v))!

@memo(array)
fn turns(c: V, m: V) -> F64:
    if c == m:
        return 0.0
    let c2 = toward(toward(c, m), m)
    if c2 == m:
        return 1.0
    1.0 + chain([m], g.next(m)).mean(of: m2 => turns(c2, m2))!

print(turns(cat, mouse), precision: 3)
