// cp: 0.1 af7fc88+dirty
// NOIP 2009 Optimal Trade: walk from city 1 to city n, buying once and selling
// once later on the walk. Best profit = max over cities v on such a walk of
// (dearest city reachable from v that still reaches n) - (cheapest city that
// 1 reaches and that reaches v).
let n: 1..=1e5 = read()
let m: 1..=5e5 = read()
domain City = 1..=n
let price: [City]1..=100 = read()
let roads: [m](u: City, v: City, ways: 1..=2) = read()

var fwd: [City]List<City> = []
var back: [City]List<City> = []
for (u, v, ways) in roads:
    fwd[u].push(v)
    back[v].push(u)
    if ways == 2:
        fwd[v].push(u)
        back[u].push(v)

// For every city, the earliest city of `order` that can reach it along adj.
fn first_reaching(adj: [City]List<City>, order: List<City>) -> [City]?City:
    var first: [City]?City = none
    var stack: List<City> = []
    for s in order:
        if first[s] == none:
            first[s] = s
            stack.push(s)
        while let u = stack.pop():
            for v in adj[u]:
                if first[v] == none:
                    first[v] = s
                    stack.push(v)
    first

let from_start = first_reaching(fwd, [City.first])
let to_end = first_reaching(back, [City.last])
let cheapest = sorted([c for c in City if from_start[c] != none], by: c => price[c])
let dearest = sorted([c for c in City if to_end[c] != none], by: c => -price[c])
let buy = first_reaching(fwd, cheapest)
let sell = first_reaching(back, dearest)
print(max(price[s] - price[b] for c in City if let b = buy[c] if let s = sell[c]) ?? 0)
