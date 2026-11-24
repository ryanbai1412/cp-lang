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
let m: n - 1..=n = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)
let adj = [V] v => g.out(v).sorted(by: a => a.to)

// the walk from town 1 that always enters the smallest unvisited neighbour,
// never using the edge `skip`
fn walk(skip: ?E) -> List<V>:
    var order: List<V> = []
    var seen: [V]Bool = false
    fn visit(u: V):
        seen[u] = true
        order.push(u)
        for a in adj[u]:
            if not seen[a.to] and Some(a.e) != skip: visit(a.to)
    visit(V.first!)
    order

// with one cycle the best walk leaves exactly one cycle edge unused
if m == n - 1: print(walk(None))
else: print(min(w for w in E.map(e => walk(e)) if w.len == n)!)
