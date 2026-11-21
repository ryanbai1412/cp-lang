// NOIP 2018 Day 2 P1, Travel
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
fn tour(skip: ?(V, V)) -> List<V>:
    var order: List<V> = []
    var seen: [V]Bool = false
    fn walk(u: V):
        seen[u] = true
        order.push(u)
        for v in adj[u]:
            if not seen[v] and skip != (u, v) and skip != (v, u): walk(v)
    walk(V.first)
    return order

if m == n - 1:
    print(tour(none))
else:
    // one cycle: the best walk leaves exactly one cycle edge unused
    print(min(t for e in edges if let t = tour((e.u, e.v)) and t.len == n)!)
