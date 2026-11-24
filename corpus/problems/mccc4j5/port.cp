// Source: Mock CCC '22 Contest 2 J5 (Dominating the Olympics), https://dmoj.ca/problem/mccc4j5
// limits: 1500 ms, 512 MB
// Problem: n <= 5000 events, event i takes 1 <= T_i <= 1e4 minutes and lists R_i < n prerequisites
//   (which may form loops; events that can never start are ignored). With unlimited
//   parallel clones, print the least time to finish every event that can be done.
// Solution: an event finishes T after its last prerequisite does, so the answer is the
//   longest weighted path in the acyclic part. Kahn's topological order driven by a min-heap
//   of finish times: an event is pushed when its last prerequisite pops (the latest finish,
//   since pops come in time order), and the last pop is the answer. O((n + R) log n).
// Bounds: n: 1..=5000; t: 1..=1e4; deps: 1..=n
// original: corpus/submissions/4264954.cpp
let n: 1..=5000 = read()
domain V = 1..=n
var time: [V]1..=1e4 = 1
var users: [V]List<V> = []
var left: [V]Int = 0
for u in V:
    let (t, deps): (1..=1e4, List<V>) = read()
    time[u] = t
    left[u] = deps.len
    for v in deps: users[v].push(u)

var pq = MinHeap<(Int, V)>()
for u in V:
    if left[u] == 0:
        pq.push((time[u], u))
var ans = 0
while let (d, u) = pq.pop():
    ans = d
    for v in users[u]:
        left[v] -= 1
        if left[v] == 0:
            pq.push((d + time[v], v))
print(ans)
