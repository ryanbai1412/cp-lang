// Source: Mock CCC '22 Contest 2 J5 (Dominating the Olympics), https://dmoj.ca/problem/mccc4j5
// limits: 1500 ms, 512 MB
// Problem: n <= 5000 events, event i takes 1 <= T_i <= 1e4 minutes and lists R_i < n prerequisites
//   (which may form loops; events that can never start are ignored). With unlimited
//   parallel clones, print the least time to finish every event that can be done.
// Solution: an event finishes T after its last prerequisite does, so the answer is the
//   longest weighted path in the acyclic part. Kahn's topological order driven by a min-heap
//   of finish times: an event is pushed when its last prerequisite pops (the latest finish,
//   since pops come in time order), and the last pop is the answer. O((n + R) log n).
// Bounds: n: 1..=5000; tasks.time: 1..=1e4
// original: corpus/submissions/4264954.cpp
let n: 1..=5000 = read()
domain V = 1..=n
let tasks: [V](time: 1..=1e4, deps: List<V>) = read()

var users: [V]List<V> = []
for u in V:
    for v in tasks[u].deps:
        users[v].push(u)
var left: [V]Int = [V] u => tasks[u].deps.len

var pq = MinHeap<(Int, V)>()
for u in V:
    if left[u] == 0:
        pq.push((tasks[u].time, u))
var ans = 0
while let (d, u) = pq.pop():
    ans = d
    for v in users[u]:
        left[v] -= 1
        if left[v] == 0:
            pq.push((d + tasks[v].time, v))
print(ans)
