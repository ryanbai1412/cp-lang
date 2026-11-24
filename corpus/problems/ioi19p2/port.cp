// Source: IOI 2019 Day 1 P2, Split the Attractions, https://dmoj.ca/problem/ioi19p2
// limits: 1000 ms, 1024 MB
// Problem: a connected graph, n <= 1e5 vertices, m <= 2e5 edges, and a + b + c = n. Split the
//   vertices into sets of sizes a, b, c, at least two of them connected; print each vertex's
//   set (1, 2, 3), or all zeros if impossible. Here stdin gives n m a b, then the edges.
// Solution: make the two smallest sets connected; the largest takes the rest. Take a spanning
//   tree and its centroid hub; a Dsu groups the hub's branches, then merges groups along edges
//   avoiding the hub while both are smaller than the small set. A group that reaches that size
//   holds it (a BFS patch from inside it); the hub with everything else stays connected and
//   fits the middle set (a BFS patch from the hub). No such group: all zeros. O((n + m) α(n)).
// original: corpus/submissions/5661848.cpp (grader function find_split)
// checker: corpus/problems/ioi19p2/checker.py
// Bounds: n: 3..=100_000; m: n - 1..=200_000; a: 1..=n - 2; b: 1..=n - 1 - a;
//   links.u, links.v: 0..=n - 1
let n: 3..=100_000 = read()
let m: n - 1..=200_000 = read()
let a: 1..=n - 2 = read()
let b: 1..=n - 1 - a = read()
domain Spot = 0..n
domain Link = 0..m
let links: [Link](u: Spot, v: Spot) = read()
let land = Graph.undirected(links)

// make the two smaller parts connected; the largest takes what is left
var by_size = [(size: a, part: 1), (size: b, part: 2), (size: n - a - b, part: 3)]
by_size.sort()
let (small, mid, big) = (by_size[0], by_size[1], by_size[2])

// group the hub's branches, and merge groups along any link for as long as both are too small
// to hold the small part
let tree = Rooted(land, Spot.first!)
let hub = tree.centroid()
var group = Dsu<Spot>()
for v in Spot:
    if let p = tree.parent(v) and v != hub and p != hub: group.unite(v, p)
for (u, v) in links:
    if u != hub and v != hub and group.size(u) < small.size and group.size(v) < small.size:
        group.unite(u, v)

// a group big enough holds the small part; everything else hangs together through the hub and
// has room for the middle part. Each part is a patch grown outwards, staying on its side.
var part: [Spot]?Int = None
fn claim(from: Spot, seed: Spot, count: Int, mark: Int):
    let inside = group.same(from, seed)
    part[from] = mark
    var (queue, taken) = (Deque.of([from]), 1)
    while let v = queue.pop_front():
        for (w, _) in land[v]:
            if taken < count and part[w].is_none and group.same(w, seed) == inside:
                part[w] = mark
                taken += 1
                queue.push(w)

if let s = Spot.find(v => v != hub and group.size(v) >= small.size):
    claim(s, s, small.size, small.part)
    claim(hub, s, mid.size, mid.part)
    print([Spot] v => part[v] ?? big.part)
else:
    print([Spot] _ => 0)
