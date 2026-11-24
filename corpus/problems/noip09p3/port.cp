// Source: NOIP 2009 Senior P3 (Optimal Trade), https://dmoj.ca/problem/noip09p3
// limits: 1000 ms, 128 MB
// Problem: n <= 1e5 cities, m <= 5e5 one-way or two-way roads, prices in 1..=100. Walk from city
//   1 to city n (repeats allowed), buying one item in some city and selling it in a later one;
//   print the largest profit, or 0 if no trade gains.
// Solution: profit = max over cities v of (dearest price on some walk v -> n) - (cheapest on
//   some walk 1 -> v). Condense the strongly connected components: the condensation is a DAG
//   in topological order, so the cheapest prefix is one forward pass from 1's component and
//   the dearest suffix one backward pass from n's. O(n + m).
// original: corpus/submissions/5185139.cpp
// Bounds: n: 1..=1e5; m: 1..=5e5; price: 1..=100; roads.u, roads.v: 1..=n; roads.ways: 1..=2
let (n, m): (1..=1e5, 1..=5e5) = read()
domain City = 1..=n
let price: [City]1..=100 = read()
let roads: [0..m](u: City, v: City, ways: 1..=2) = read()

var road: [City]List<City> = []
for (u, v, ways) in roads:
    road[u].push(v)
    if ways == 2: road[v].push(u)

// inside a strongly connected component every price is on a walk, so work on the condensation
let (domain C, comp) = scc(road)
let dag = condense(road, comp)
var least: [C]?Int = None
var most: [C]?Int = None
for v in City:
    least[comp[v]] min= price[v]
    most[comp[v]] max= price[v]

// low[c]: cheapest price on a walk 1 -> c; high[c]: dearest on a walk c -> n
var low: [C]?Int = None
low[comp[City.first!]] = least[comp[City.first!]]
for c in C:
    if let l = low[c]:
        for e in dag[c]: low[e.to] min= min(l, least[e.to]!)
var high: [C]?Int = None
high[comp[City.last!]] = most[comp[City.last!]]
for c in C.rev:
    for e in dag[c]:
        if let h = high[e.to]: high[c] max= max(h, most[c]!)

var profit = 0
for c in C:
    if let l = low[c] and let h = high[c]: profit max= h - l
print(profit)
