// CCC 2018 S5 Maximum Strategic Savings: n planets with m cities each. A
// flight joins two cities on every planet, a portal joins two planets in
// every city. Print the most that can be saved by closing links while every
// city on every planet stays connected (total cost minus the spanning tree).
let (n, m, p, q): (1..=1e5, 1..=1e5, 1..=1e5, 1..=1e5) = read()
domain Planet = 1..=n
domain City = 1..=m
let flights: [0..p](a: City, b: City, cost: 1..=1e8) = read()       // one copy per planet
let portals: [0..q](a: Planet, b: Planet, cost: 1..=1e8) = read()   // one copy per city

enum Link:
    Flight(a: City, b: City)
    Portal(a: Planet, b: Planet)
var links: List<(cost: Int, link: Link)> = [(f.cost, Flight(f.a, f.b)) for f in flights]
for g in portals: links.push((g.cost, Portal(g.a, g.b)))

// Kruskal on the product graph without building it: a flight that joins two
// groups of cities is kept once for each group of planets, and vice versa.
var cities = Dsu<City>()
var planets = Dsu<Planet>()
var kept = 0
for (cost, link) in sorted(links, by: l => l.cost):
    match link:
        Flight(a, b) => if cities.unite(a, b): kept += cost * planets.components
        Portal(a, b) => if planets.unite(a, b): kept += cost * cities.components
print(n * sum(f.cost for f in flights) + m * sum(g.cost for g in portals) - kept)
