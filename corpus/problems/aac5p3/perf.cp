// AAC 5 P3, tuned: port.cp's intervals pushed down a breadth-first order of the adjacency lists,
// each list reserved to its degree and sorted in place once its parent's edge is dropped, as the
// original does, instead of building a `Rooted` tree and a sorted copy of every node's children.
let (n, k): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let hills: [n - 1](u: V, v: V, d: 1..=1e9) = read()
let skills = read<[k]1..=1e9>().sorted()

var adj: [V]List<(to: V, d: 1..=1e9)> = []
var degree: [V]0..=2e5 = 0
for (u, v, _) in hills: (degree[u], degree[v]) = (degree[u] + 1, degree[v] + 1)
for v in V: adj[v].reserve(degree[v])
for (u, v, d) in hills:
    adj[u].push((to: v, d:))
    adj[v].push((to: u, d:))

// a breadth-first order from the root, then each node's list without its parent's edge
var order = [V.first!]
var up: [V]?V = None
var at = 0
while at < order.len:
    let v = order[at]
    at += 1
    for e in adj[v]:
        if Some(e.to) != up[v]:
            up[e.to] = v
            order.push(e.to)
for v in V:
    if let p = up[v]:
        let j = (0..adj[v].len).first(j => adj[v][j].to == p)!
        adj[v][j] = adj[v].last!
        adj[v].pop()

// the skills that reach v, a half-open interval
var reach_lo: [V]0..=1e9 + 1 = 0
var reach_hi: [V]0..=1e9 + 1 = 0
reach_hi[V.first!] = 1e9 + 1
for v in order:
    adj[v].sort(by: e => e.d)
    let down = adj[v]
    for (i, e) in down.items:
        // a skill goes to the closer of two neighbouring hills, the easier one on a tie
        let lo = if i > 0: (down[i - 1].d + e.d) / 2 + 1 else: reach_lo[v]
        let hi = if i + 1 < down.len: (e.d + down[i + 1].d) / 2 + 1 else: reach_hi[v]
        reach_lo[e.to] = max(lo, reach_lo[v])
        reach_hi[e.to] = max(lo, min(hi, reach_hi[v]))
for v in V:
    print(skills.count_in(reach_lo[v]..reach_hi[v]))
