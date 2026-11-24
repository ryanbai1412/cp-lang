// Source: EGOI 2022 Day 1 Social Engineering, https://dmoj.ca/problem/egoi22p3
// Problem: interactive: a connected simple graph, n <= 2e5, m <= 4e5, Maria at vertex 1. Moves
//   walk along unused edges, Maria choosing hers and you everyone else's; whoever cannot move
//   loses. Give up at once if Maria wins, else play until she has no move.
// Solution: Maria loses iff her friends (neighbours) pair up by edge-disjoint paths avoiding
//   her: then each move into a friend goes along its path to its partner and back to her. That
//   holds iff every component of the graph without her has an even number of friends (an odd
//   degree leaves some component odd). On a spanning forest of the graph without her, bottom
//   up each vertex pairs its open friends through itself (the tree path between them) and
//   passes at most one up; a root left with one is an odd component. O((n + m) log n).
// Stdin protocol (the grader played over stdin, as in spec.cp): n, m, then m edges (u, v) in
//   1..=n. Prints "!" at once when the strategy gives up. Otherwise asks "?" for Maria's move (0
//   once she resigns), answers a move into a friend along its paired path and back to vertex
//   1, and prints "!" once she has resigned.
// Bounds: n: 2..=2e5; m: 1..=4e5; edges.u, edges.v: 1..=n
// original: corpus/submissions/5199301.cpp

let wire = Wire()
let n: 2..=2e5 = read()
let m: 1..=4e5 = read()
domain V = 1..=n
let edges: [m](u: V, v: V) = read()

let g = Graph.undirected(edges)
let home = V.first!
var friend: [V]Bool = false
for v in g.next(home): friend[v] = true
let forest = Rooted.forest(g.only(keep: v => v != home, from: v => v != home))
var route: [V]List<V> = []  // a friend's path to its partner

fn pair_up(u: V) -> ?V:  // the friend under u left open, if any
    var open: List<V> = if friend[u]: [u] else: []
    for c in forest.children(u):
        if let x = pair_up(c): open.push(x)
    for i in 0..open.len / 2:
        let (a, b) = (open[2 * i], open[2 * i + 1])
        route[a] = forest.path(a, b)
        route[b] = forest.path(b, a)
    if open.len % 2 == 1: open.last else: None

if any(pair_up(r) != None for r in forest.roots if r != home):
    print("!")
else:
    while true:
        let move: 0..=n = wire.ask("?")
        if move == 0: break
        for v in route[V(move)].drop(1): print(v)
        print(home)
    print("!")
