// AAC 5 P3, tuned: port.cp's intervals pushed down a breadth-first order, with every node's hills
// packed easiest first into one array (the hills sorted by difficulty once, then placed by
// counting), instead of a `Rooted` tree and a sorted copy of every node's children.
let (n, k): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
var hills: [n - 1](u: V, v: V, d: 1..=1e9) = read()
let skills = read<[k]1..=1e9>().sorted()
hills.sort(by: h => h.d)

// node v's hills sit at first[v]..next[v] of `to` and `by`, easiest first
var degree: [V]0..=2e5 = 0
for (u, v, _) in hills: (degree[u], degree[v]) = (degree[u] + 1, degree[v] + 1)
var next: [V]0..=4e5 = 0
var at = 0
for v in V:
    next[v] = at
    at += degree[v]
let first = next.clone()
var to: [2 * (n - 1)]V = V.first!
var by: [2 * (n - 1)]1..=1e9 = 1
for (u, v, d) in hills:
    (to[next[u]], by[next[u]]) = (v, d)
    next[u] += 1
    (to[next[v]], by[next[v]]) = (u, d)
    next[v] += 1

// a breadth-first order from the root
var order = [V.first!]
var up: [V]?V = None
var i = 0
while i < order.len:
    let v = order[i]
    i += 1
    for j in first[v]..next[v]:
        if Some(to[j]) != up[v]:
            up[to[j]] = v
            order.push(to[j])

// the skills that reach v, a half-open interval
var reach_lo: [V]0..=1e9 + 1 = 0
var reach_hi: [V]0..=1e9 + 1 = 0
reach_hi[V.first!] = 1e9 + 1
for v in order:
    var prev: ?Int = None
    for j in first[v]..next[v]:
        if Some(to[j]) == up[v]: continue
        let after = if j + 1 < next[v] and Some(to[j + 1]) == up[v]: j + 2 else: j + 1
        // a skill goes to the closer of two neighbouring hills, the easier one on a tie
        let lo = prev.map(d => (d + by[j]) / 2 + 1) ?? reach_lo[v]
        let hi = if after < next[v]: (by[j] + by[after]) / 2 + 1 else: reach_hi[v]
        reach_lo[to[j]] = max(lo, reach_lo[v])
        reach_hi[to[j]] = max(lo, min(hi, reach_hi[v]))
        prev = Some(by[j])
for v in V:
    print(skills.count_in(reach_lo[v]..reach_hi[v]))
