// Source: CCO 2024 P1 (Treasure Hunt), https://dmoj.ca/problem/cco24p1
// Problem: an undirected graph of N <= 1e6 islands, island i holding v_i coins, and M <= 1e6
//   routes of cost c_i (0 <= v_i, c_i <= 1e9). For every start x print the best net profit,
//   the max over y of v_y - dist(x, y) (y = x allowed).
// Solution: the answer at x is -min over y of (-v_y + dist(y, x)), a shortest path from a
//   virtual source joined to each y by weight -v_y. So one `dijkstra` starts every island y
//   at -v_y (route costs are nonnegative). O((N + M) log M).
let (n, m): (1..=1e6, 0..=1e6) = read()
domain Island = 1..=n
domain Route = 0..m
let coins: [Island]0..=1e9 = read()
let routes: [Route](u: Island, v: Island, w: 0..=1e9) = read()
let paths = dijkstra(Graph.undirected(routes), from: Island.map(y => (y, -coins[y])))
for x in Island: print(-paths[x]!)
