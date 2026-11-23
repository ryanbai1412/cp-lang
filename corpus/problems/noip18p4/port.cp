// Source: NOIP 2018 Day 2 P1 (Travel), https://dmoj.ca/problem/noip18p4
// Problem: a connected graph on n <= 5000 towns with m = n - 1 or m = n roads. Walk from any
//   start, each step entering an unvisited neighbour or backing up the edge you came in by;
//   print the lexicographically smallest order in which all towns are first visited.
// Solution: start at town 1 and DFS into the smallest unvisited neighbour first (sorted
//   adjacency). A tree needs one DFS. With one cycle the walk leaves one cycle edge unused,
//   so delete each edge in turn, keep the walks that reach all n towns and print the
//   smallest: O(n m).
// original: corpus/submissions/4004140.cpp
let n: 1..=5000 = read()
let m: 1..=n = read()
domain V = 1..=n
let edges: [m](u: V, v: V) = read()

var adj: [V]List<V> = []
for e in edges:
    adj[e.u].push(e.v)
    adj[e.v].push(e.u)
for v in V: adj[v].sort()

// the walk from town 1 that always enters the smallest unvisited neighbour,
// never using the edge `skip`
var order: List<V> = []
var seen: [V]Bool = false
var skip_u = V(1)
var skip_v = V(1)
fn walk(u: V):
    seen[u] = true
    order.push(u)
    for v in adj[u]:
        let skipped = (u == skip_u and v == skip_v) or (u == skip_v and v == skip_u)
        if not seen[v] and not skipped: walk(v)

if m == n - 1:
    walk(V(1))
    print(order)
else:
    // one cycle: the best walk leaves exactly one cycle edge unused
    var best: List<V> = []
    for e in edges:
        order = []
        for v in V: seen[v] = false
        skip_u = e.u
        skip_v = e.v
        walk(V(1))
        if order.len == n and (best.len == 0 or order < best): best = order.clone()
    print(best)
