// cp: 0.1 0f8d77b+dirty
// IOI 2011 Crocodile's Underground City: escape from chamber 0 to any exit
// through weighted corridors. At every chamber the gatekeeper blocks one
// corridor after seeing where you stand. Minimise the worst-case time.
let n: 3..=1e5 = read()
let m: 2..=1e6 = read()
let k: 1..=n = read()
domain Chamber = 0..n
let corridors: [m](u: Chamber, v: Chamber, time: 1..=1e9) = read()
let exits: [k]Chamber = read()
var maze: [Chamber]List<(Chamber, Int)> = []
for (u, v, time) in corridors:
    maze[u].push((v, time))
    maze[v].push((u, time))

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
    if arrivals[u] == 2:
        if u == Chamber.first:
            print(d)
            break
        for (v, time) in maze[u]: pq.push((d + time, v))
