// Source: DMOPC '21 Contest 9 P3 (Terrible Trains), https://dmoj.ca/problem/dmopc21c9p3
// Problem: a connected graph of n <= 3000 stations and m <= 3000 one-hour routes. Each of q <= 1e6
//   commuters asks: after adding one new one-hour route of our choice, can one go S -> T within X
//   hours and U -> V within Y hours? Print YES or NO for each.
// Solution: all-pairs distances by a BFS from every station, O(n (n + m)). If one trip already
//   fits, the new route joins the other trip's ends. Otherwise both trips use the new route (p,
//   q); cutting a shortest S-U path and a shortest T-V path, that works iff d(S,U) + d(T,V) <= X +
//   Y - 2 (or the same with U and V swapped). O(1) per query.
// original: corpus/submissions/4564027.cpp
// All-pairs distances by one BFS per source (the graph is connected).

let n: 1..=3000 = read()
let m: 1..=3000 = read()
domain Station = 1..=n
let routes: [m](u: Station, v: Station) = read()

let g = Graph.undirected(routes)
var dist: [Station, Station]0..n = 0  // the graph is connected
for from in Station:
    for (v, d) in bfs(g, from: [from]).iter(): dist[from, v] = d

let q: 1..=1e6 = read()
for _ in 0..q:
    let (s, t, x): (Station, Station, Int) = read()
    let (u, v, y): (Station, Station, Int) = read()
    // a trip that fits leaves the new route to the other; else both share it
    let fits = dist[s, t] <= x or dist[u, v] <= y or
        min(dist[s, u] + dist[t, v], dist[s, v] + dist[t, u]) <= x + y - 2
    print(yes(fits))
