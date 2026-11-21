// IOI 2011 Crocodile's Underground City: escape from chamber 0 to any exit
// through weighted corridors. At every chamber the gatekeeper blocks one
// corridor after seeing where you stand. Minimise the worst-case time.
let (n, m, k): (3..=1e5, 2..=1e6, 1..=1e5) = read()
domain Chamber = 0..n
domain Corridor = 0..m
let corridors: [Corridor](u: Chamber, v: Chamber, time: 1..=1e9) = read()
let exits: [0..k]Chamber = read()
let maze = Graph.undirected(corridors)

// The best corridor out of a chamber is the one that gets blocked, so a
// chamber's time is its second-cheapest way out: Dijkstra from the exits,
// settling each chamber the second time it comes off the heap.
var arrivals: [Chamber]Int = 0
var pq = MinHeap<(Int, Chamber)>()
for x in exits:
    arrivals[x] = 1
    pq.push((0, x))
while let (d, u) = pq.pop():
    arrivals[u] += 1
    if arrivals[u] != 2: continue
    if u == Chamber.first:
        print(d)
        return
    for (v, c) in maze.out(u): pq.push((d + corridors[c].time, v))
