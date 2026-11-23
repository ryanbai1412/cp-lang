// CSES 1666 Building Roads: add the fewest roads so every city is reachable from every other.
// In: n m, then m roads "a b". Out: k, then k new roads "a b".
domain City = 1..=read<1..=1e5>()
let m = read<1..=2e5>()
var cities = Dsu<City>()
for r in read<[m](u: City, v: City)>(): cities.unite(r.u, r.v)
print(cities.components - 1)
for (a, b) in cities.roots().adjacent: print(a, b)
