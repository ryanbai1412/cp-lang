// Source: NOIP 2009 Senior P3 (Optimal Trade), https://dmoj.ca/problem/noip09p3
// Problem: n <= 1e5 cities, m <= 5e5 one-way or two-way roads, prices in 1..=100. Walk from city
//   1 to city n (repeats allowed), buying one item in some city and selling it in a later one;
//   print the largest profit, or 0 if no trade gains.
// Solution: profit = max over cities v of (dearest price on some path v -> n) - (cheapest on
//   some path 1 -> v). Both come from a queue-based relaxation (SPFA-style, by min and by max)
//   until stable: forward from 1, and on reversed roads from n. A label only improves within
//   1..=100, so each city re-enters the queue at most 100 times: O(100 (n + m)).
let (n, m): (1..=1e5, 1..=5e5) = read()
domain City = 1..=n
let price: [City]1..=100 = read()
let roads: [0..m](u: City, v: City, ways: 1..=2) = read()

var fwd: [City]List<City> = []
var back: [City]List<City> = []
for (u, v, ways) in roads:
    fwd[u].push(v)
    back[v].push(u)
    if ways == 2:
        fwd[v].push(u)
        back[u].push(v)

// For every city `from` reaches along adj: the best price (by `pick`) met on the way.
fn spread(adj: [City]List<City>, from: City, pick: (Int, Int) -> Int) -> [City]?Int:
    var best: [City]?Int = None
    var queue = Deque<City>()
    best[from] = price[from]
    queue.push(from)
    while let u = queue.pop_front():
        for v in adj[u]:
            let via = pick(best[u]!, price[v])
            if best[v] != pick(best[v] ?? via, via):
                best[v] = via
                queue.push(v)
    best

let cheapest = spread(fwd, City.first!, min)
let dearest = spread(back, City.last!, max)
var profit = 0
for c in City: profit max= dearest[c] - cheapest[c]
print(profit)
