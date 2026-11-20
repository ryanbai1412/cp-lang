// NOIP 2009 Optimal Trade: walk from city 1 to city n, buying once and selling
// once later on the walk. Best profit = max over cities v on such a walk of
// (dearest price from v on) - (cheapest price up to v). Relaxing prices along
// roads until nothing changes handles the cycles.
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

// For every city `from` reaches along adj: the best price (smallest key) met on the way.
fn spread(adj: [City]List<City>, from: City, key: Int -> Int) -> [City]?Int:
    var best: [City]?Int = none
    var queue = Deque<City>()
    best[from] = key(price[from])
    queue.push(from)
    while let u = queue.pop_front():
        for v in adj[u]:
            let via = min(best[u]!, key(price[v]))
            if best[v] == none or via < best[v]!:
                best[v] = via
                queue.push(v)
    best

let cheapest = spread(fwd, City.first, p => p)
let dearest = spread(back, City.last, p => -p)
var profit = 0
for c in City: profit max= -dearest[c] - cheapest[c]
print(profit)
