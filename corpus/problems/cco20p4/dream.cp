# CCO 2020 P4 Travelling Salesperson: every pair of cities is joined by a red or blue road.
# For each start, give a closed walk visiting every city that changes colour at most once.
# One of the two colour graphs of a complete graph is connected; walk its DFS tree there and back
# (an Euler tour, a single colour) and rotate it to each start.
enum Colour = 'R' Red | 'B' Blue
domain V = [read Int]
let road = read lower_triangle [V, V]Colour   # row v lists v's roads to earlier cities
let tour = [Red, Blue].first_some(c =>
    let t = dfs(u => V.filter(v => v != u and road[u, v] == c), from: V.first).euler_tour()
    if t.distinct().len == V.count: Some(t.but_last) else: None)!
for v in V:
    let walk = tour.rotated(to: tour.index_of(v)!)
    print(walk.len)
    print(walk)
