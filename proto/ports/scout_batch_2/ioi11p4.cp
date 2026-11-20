// IOI 2011 Crocodile: escape from chamber 0 to any exit through corridors
// of known length. Each time you enter a chamber, the crocodile blocks one
// corridor out of it. Print the time the escape takes in the worst case.
//
// The best way out of a chamber is always the one blocked, so a chamber is
// worth its second-best way out. Run Dijkstra from the exits and settle a
// chamber the second time it comes off the heap.
use std.graph.{Graph}
use std.heap.{MinHeap}

let (n, m, k): (Int, Int, Int) = read()
domain Room = 0..n
domain Hall = 0..m
domain X = 0..k
let halls: [Hall](u: Room, v: Room, len: Int) = read()
let exits: [X]Room = read()
let g = Graph<Room, Hall>.undirected(halls)

var offers: [Room]Int = 0                   // ways out seen so far, cheapest first
var heap = MinHeap<(Int, Room)>()
for x in exits:
    offers[x] = 1                           // an exit needs no second way out
    heap.push((0, x))
while let (t, u) = heap.pop():
    offers[u] += 1
    if offers[u] != 2: continue
    if u == Room.first:
        print(t)
        break
    for a in g.adj(u):
        if offers[a.to] < 2: heap.push((t + halls[a.e].len, a.to))
