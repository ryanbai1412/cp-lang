// CCC 2018 S5 Maximum Strategic Savings: n planets with m cities each. A
// flight joins two cities on every planet, a portal joins two planets in
// every city. Print the most that can be saved by closing links while every
// city on every planet stays connected (total cost minus the spanning tree).
use std.dsu.{Dsu}                                                   // WISH F1

let n: 1..=1e5 = read()                     // planets
let m: 1..=1e5 = read()                     // cities on each planet
let p: 1..=1e5 = read()
let q: 1..=1e5 = read()
domain Planet = 1..=n
domain City = 1..=m
let flights: [p](a: City, b: City, cost: 1..=1e8) = read()          // one copy per planet
let portals: [q](a: Planet, b: Planet, cost: 1..=1e8) = read()      // one copy per city

// Kruskal on the product graph without building it: a flight that joins two
// groups of cities is kept once for each group of planets, and vice versa.
var cities = Dsu<City>.new()
var planets = Dsu<Planet>.new()
var kept = 0
for link in merge(sorted(flights, by: .cost), sorted(portals, by: .cost), by: .cost):   // WISH F12, F14
    match link:
        Left(f) => if cities.unite(f.a, f.b): kept += f.cost * planets.count
        Right(g) => if planets.unite(g.a, g.b): kept += g.cost * cities.count
print(n * sum(f.cost for f in flights) + m * sum(g.cost for g in portals) - kept)
