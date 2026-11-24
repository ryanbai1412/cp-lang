// Source: CCC 2018 S5 (Maximum Strategic Savings), https://dmoj.ca/problem/ccc18s5
// Problem: n planets with m cities each (n, m <= 1e5). p <= 1e5 flights each join cities a, b
//   on every planet, q <= 1e5 portals each join planets x, y at every city, costs <= 1e8.
//   Print the most total cost that can be cut while all n m cities stay connected.
// Solution: the answer is the total cost minus a minimum spanning tree of the product graph.
//   Kruskal over the p + q link kinds by cost with one Dsu for cities and one for planets:
//   a flight that joins two city groups is kept once per current planet group, a portal
//   once per current city group. O((p + q) log).
// original: corpus/submissions/4329864.cpp
let (n, m, p, q): (1..=1e5, 1..=1e5, 1..=1e5, 1..=1e5) = read()
domain Planet = 1..=n
domain City = 1..=m
domain F = 0..p
domain G = 0..q
let flights: [F](a: City, b: City, cost: 1..=1e8) = read()
let portals: [G](a: Planet, b: Planet, cost: 1..=1e8) = read()

enum Link:
    Flight(f: F)
    Portal(g: G)
let links = [(flights[f].cost, Flight(f)) for f in F] + [(portals[g].cost, Portal(g)) for g in G]

var cities = Dsu<City>()
var planets = Dsu<Planet>()
var kept = 0
for (cost, link) in links.sorted(by: l => l.0):
    match link:
        Flight(f) => if cities.unite(flights[f].a, flights[f].b): kept += cost * planets.components
        Portal(g) => if planets.unite(portals[g].a, portals[g].b): kept += cost * cities.components
print(n * sum(f.cost for f in flights) + m * sum(g.cost for g in portals) - kept)
