// NOI 2003 P4, the detour: longest dist(c, nearer of a/b) + dist(a, b) on a weighted tree
// original: corpus/submissions/5032732.cpp
let (n, m): (1..=2e5, 0..=2e5) = read()
domain Home = 1..=n
let roads: [0..m](u: Home, v: Home, w: 1..=1e9) = read()
let g = Graph.undirected(roads)

// a and b are the two ends of a diameter, and the third home is whoever is
// farthest from the nearer of the two
fn farthest(d: [Home]?Int) -> Home: max_by(Home, v => d[v]!)!
let a = farthest(dijkstra(g, from: Home.first))
let da = dijkstra(g, from: a)
let b = farthest(da)
let db = dijkstra(g, from: b)
print(da[b]! + max(min(da[c]!, db[c]!) for c in Home)!)
