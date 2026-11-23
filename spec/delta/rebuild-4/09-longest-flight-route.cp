// CSES 1680 Longest Flight Route
// in: n m, then m one-way flights a b (no cycles); out: the most cities on a route 1 -> n and the route, or IMPOSSIBLE
let (n, m) = read<(Int, Int)>()
domain City = 1..=n
let g = Graph.directed(read<[m](u: City, v: City)>())
var cities: [City]?Int = None
var from: [City]?City = None
cities[City.first!] = 1
for u in topo(g)!:
    if let c = cities[u]:
        for a in g.out(u):
            if cities[a.to] max= c + 1: from[a.to] = u
if let c = cities[City.last!]:
    print(c)
    print(from.chain(City.last!).collect().rev)
else:
    print("IMPOSSIBLE")
