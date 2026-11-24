// Source: CCC 2018 S5 (Maximum Strategic Savings), https://dmoj.ca/problem/ccc18s5
// Problem: n planets with m cities each (n, m <= 1e5). p <= 1e5 flights each join cities a, b
//   on every planet, q <= 1e5 portals each join planets x, y at every city, costs <= 1e8.
//   Print the most total cost that can be cut while all n m cities stay connected.
// Solution: the answer is the total cost minus a minimum spanning tree of the product graph.
//   Kruskal over the p + q link kinds by cost with one Dsu for cities and one for planets:
//   a flight that joins two city groups is kept once per current planet group, a portal
//   once per current city group. O((p + q) log).
let (n, m, p, q): (1..=1e5, 1..=1e5, 1..=1e5, 1..=1e5) = read()
domain Planet = 1..=n
domain City = 1..=m
let flights: [0..p](a: City, b: City, cost: 1..=1e8) = read()      // one copy per planet
let portals: [0..q](a: Planet, b: Planet, cost: 1..=1e8) = read()  // one copy per city

enum Link:
    Flight(a: City, b: City)
    Portal(a: Planet, b: Planet)
let links: List<(cost: Int, link: Link)> = [(f.cost, Flight(f.a, f.b)) for f in flights] + [
    (g.cost, Portal(g.a, g.b)) for g in portals
]

// Kruskal on the product graph without building it: a flight that joins two
// groups of cities is kept once for each group of planets, and vice versa.
var cities = Dsu<City>()
var planets = Dsu<Planet>()
var kept = 0
for (cost, link) in links.sorted(by: l => l.cost):
    match link:
        Flight(a, b) => if cities.unite(a, b): kept += cost * planets.components
        Portal(a, b) => if planets.unite(a, b): kept += cost * cities.components
print(n * sum(f.cost for f in flights) + m * sum(g.cost for g in portals) - kept)
