// Source: COCI 2021/2022 Contest 6 #3 (Naboj), https://dmoj.ca/problem/coci21c6p3
// limits: 1000 ms, 512 MB
// Problem: a connected simple graph, n <= 2e5 balls and 1 <= m <= 5e5 wires, each wire
//   wanting its electrons at a given end. Charging a ball + pulls every wire at it towards
//   it, - pushes them away; the last charge on a wire's ends decides it. Print -1 or a sequence of k <= 2e5 chargings.
// Solution: think backwards: the last ball charged must have all its remaining wires pointing
//   the same way (in- or out-degree 0), and its wires are then settled. Peel such balls with a
//   queue, updating degrees, like a topological sort; if some ball is never peeled the wires
//   hold a cycle and the answer is -1. Print the peel order reversed, O(n + m).
// original: corpus/submissions/4530006.cpp
// Bounds: n: 1..=2e5; m: 1..=5e5; u, v: 1..=n
let n: 1..=2e5 = read()
let m: 1..=5e5 = read()
domain V = 1..=n

var out_adj: [V]List<V> = []
var in_adj: [V]List<V> = []
var in_deg: [V]Int = 0
var out_deg: [V]Int = 0

for _ in 0..m:
    let u: V = read()
    let v: V = read()
    out_adj[u].push(v)
    in_adj[v].push(u)
    out_deg[u] += 1
    in_deg[v] += 1

var vis: [V]Bool = false
var q: List<V> = []
var ops: List<(V, Int)> = []

fn try_push(mut seen: [V]Bool, mut que: List<V>, u: V):
    if seen[u]:
        return
    if in_deg[u] == 0 or out_deg[u] == 0:
        seen[u] = true
        que.push(u)

for u in V:
    try_push(mut vis, mut q, u)

while q.len > 0:
    let u = q.pop()!
    if in_deg[u] > 0:
        ops.push((u, 0))
    elif out_deg[u] > 0:
        ops.push((u, 1))
    else:
        ops.push((u, 0))
    for idx in 0..out_adj[u].len:
        let w = out_adj[u][idx]
        in_deg[w] -= 1
        try_push(mut vis, mut q, w)
    for idx in 0..in_adj[u].len:
        let w = in_adj[u][idx]
        out_deg[w] -= 1
        try_push(mut vis, mut q, w)

var all_done = true
for u in V:
    if not vis[u]:
        all_done = false

if not all_done:
    print(-1)
else:
    print(ops.len)
    var i = ops.len - 1
    while i >= 0:
        let op = ops[i]
        print(op.0, op.1)
        i -= 1
