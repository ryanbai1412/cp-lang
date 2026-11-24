// Tuned: Dinic written out over flat arc arrays, grouped by tail once all arcs are in,
// with 16-bit node ids, where FlowGraph keeps a growing list of 64-bit arcs per node.
//
// Source: NOI 2006 P4 (Maximum Profit), https://dmoj.ca/problem/noi06p4
// limits: 1000 ms, 256 MB
// Problem: N <= 5000 candidate stations with build costs P_i and M <= 50000 user groups;
//   group i pays C_i if both stations A_i and B_i are built. Print the maximum of the total
//   payments minus the total build cost.
// Solution: a maximum-weight closure as a min cut: source -> group (C_i), group -> its two
//   stations (uncapped, never cut), station -> sink (P_i). A cut either drops a group or
//   pays for its stations, so the answer is sum C_i minus the max flow of the network.
// original: corpus/submissions/5032812.cpp
// Bounds: n: 1..=5000; m: 1..=50000; users.a, users.b: 1..=n
let (n, m): (1..=5000, 1..=50000) = read()
domain Station = 1..=n
domain User = 1..=m
let cost: [Station]Int = read()
let users: [User](a: Station, b: Station, pay: Int) = read()

// the network's nodes: the stations, then the groups, then source and sink; arc e's
// reverse is e ^ 1
let nodes = n + m + 2
let (source, sink) = (n + m, n + m + 1)
let arcs = 2 * (n + 3 * m)
var tail: [arcs]0..=55001 = 0
var head: [arcs]0..=55001 = 0
var left: [arcs]Int = 0
var added = 0
fn add(u: 0..=55001, v: 0..=55001, cap: Int):
    tail[added] = u
    head[added] = v
    left[added] = cap
    tail[added + 1] = v
    head[added + 1] = u
    added += 2
for s in Station: add(s.ord, sink, cost[s])
for u in User:
    add(source, n + u.ord, users[u].pay)
    add(n + u.ord, users[u].a.ord, 1 << 60)
    add(n + u.ord, users[u].b.ord, 1 << 60)

// out[first[v]..first[v + 1]] are v's arcs, in the order they were added
var first: [nodes + 1]0..=310000 = 0
for e in 0..arcs: first[tail[e] + 1] += 1
for v in 0..nodes: first[v + 1] += first[v]
var out: [arcs]0..=310000 = 0
var fill: [nodes]0..=310000 = [first[v] for v in 0..nodes]
for e in 0..arcs:
    out[fill[tail[e]]] = e
    fill[tail[e]] += 1

var level: [nodes]-1..=55002 = -1
var next: [nodes]0..=310000 = 0
var queue: [nodes]0..=55001 = 0
var path: List<0..=310000> = []
var flow = 0
unchecked:
    while true:
        for v in 0..nodes: level[v] = -1
        level[source] = 0
        queue[0] = source
        var (at, end) = (0, 1)
        while at < end:
            let u = queue[at]
            at += 1
            for i in first[u]..first[u + 1]:
                let v = head[out[i]]
                if left[out[i]] > 0 and level[v] < 0:
                    level[v] = level[u] + 1
                    queue[end] = v
                    end += 1
        if level[sink] < 0: break
        for v in 0..nodes: next[v] = first[v]
        // one augmenting path in the level graph per round, found without recursion
        var stuck = false
        while not stuck:
            var u = source
            while u != sink:
                if next[u] < first[u + 1]:
                    let e = out[next[u]]
                    if left[e] > 0 and level[head[e]] == level[u] + 1:
                        path.push(e)
                        u = head[e]
                    else:
                        next[u] += 1
                elif path.is_empty:
                    stuck = true
                    break
                else:
                    u = tail[path.pop()!]
                    next[u] += 1
            if stuck: break
            var amount = 1 << 62
            for e in path: amount = min(amount, left[e])
            for e in path:
                left[e] -= amount
                left[e ^ 1] += amount
            flow += amount
            while not path.is_empty: path.pop()
print(users.sum(u => u.pay) - flow)
