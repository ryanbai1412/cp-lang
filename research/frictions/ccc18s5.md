# Friction Report: ccc18s5

## An enum of domain pairs overflowed its packed `int` (fixed in Core)
- **Where:** `enum Link: Flight(a: City, b: City) / Portal(a: Planet, b: Planet)`, sizes 1e5.
- **What felt wrong:** codegen packed the enum into a C++ `int` over a 2e10 range: `--checks all`
  failed "no arm of this `match` fits the value" at `gen.py --max`; the unchecked build was UB.
  The port carries flight and portal indexes instead (`Flight(f: F)`), which also reads well.
- **Goal it hurts:** correctness.
- **Ideal code:** either spelling.
- **General fix:** Core's packing width table (reported with a repro, fixed upstream).
- **Severity:** high

## Kruskal over a product graph is hand-rolled
- **Where:** the merged sort and two `Dsu`s.
- **What felt wrong:** `kruskal` takes one edge list; the product-graph variant (each kind weighted
  by the other side's component count) is written out.
- **Goal it hurts:** brevity.
- **Ideal code:** see the dream; `sorted_merge` of the two kinds by cost removes the enum.
- **General fix:** none needed beyond a merged-by-key iterator over two sorted sequences.
- **Severity:** low

## Dream

```cp
let (n, m, p, q): (1..=1e5, 1..=1e5, 1..=1e5, 1..=1e5) = read()
domain Planet = 1..=n
domain City = 1..=m
let flights: [p](a: City, b: City, cost: 1..=1e8) = read()
let portals: [q](a: Planet, b: Planet, cost: 1..=1e8) = read()
var cities = Dsu<City>()
var planets = Dsu<Planet>()
let kept = sum(merge(flights.map(Left), portals.map(Right), by: e => e.cost).map(e => match e:
    Left(f) => if cities.unite(f.a, f.b): f.cost * planets.components else: 0
    Right(g) => if planets.unite(g.a, g.b): g.cost * cities.components else: 0))
print(n * sum(f.cost for f in flights) + m * sum(g.cost for g in portals) - kept)
```

## Measurements

- Tokens: port 325, original 787
- Max input (`gen.py --max`): port 0.03 s, 15036 KB, original 0.04 s, 9872 KB
- Emitted C++: 18473 bytes
