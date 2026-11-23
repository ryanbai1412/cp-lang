// Source: IOI 2011 P4 (Crocodile's Underground City), https://dmoj.ca/problem/ioi11p4
// Problem: N <= 1e5 chambers, M <= 1e6 corridors with lengths <= 1e9, K exit chambers, start at
//   0. Each time she leaves a chamber the gatekeeper may block one corridor out of it. Print the
//   smallest worst-case time to reach an exit. Grader task; here stdin is n m k, the corridors
//   (u v len), then the exits.
// Solution: the best way out of a chamber is always blocked, so a chamber is worth its
//   second-best (neighbour's worth + corridor). Dijkstra from all exits, settling a chamber the
//   second time it comes off the heap; stop when chamber 0 settles. O((N + M) log M).

let (n, m, k): (Int, Int, Int) = read()
domain Room = 0..n
domain Hall = 0..m
domain X = 0..k
let halls: [Hall](u: Room, v: Room, len: Int) = read()
let exits: [X]Room = read()
let g = Graph<Room, Hall>.undirected(halls)

var offers: [Room]Int = 0  // ways out seen so far, cheapest first
var heap = MinHeap<(Int, Room)>()
for x in exits:
    offers[x] = 1  // an exit needs no second way out
    heap.push((0, x))
while let (t, u) = heap.pop():
    offers[u] += 1
    if offers[u] != 2: continue
    if u == Room.first:
        print(t)
        break
    for a in g.out(u):
        if offers[a.to] < 2: heap.push((t + halls[a.e].len, a.to))
