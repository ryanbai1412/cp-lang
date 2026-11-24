# Friction Report: ccc18s5

## Links are packed into one `Int` to sort them together
- **Where:** `[flights[f].cost << 18 | f.ord for f in F] + [portals[g].cost << 18 | p + g.ord for g in G]`,
  sorted, then unpacked with `key >> 18` and `key % (1 << 18)`.
- **What felt wrong:** a C++ sort trick: the two link kinds share one sorted `Int` list so the
  walk sees them by cost. An earlier `enum Link` spelling overflowed Core's packed `int`
  (`--checks all` failed "no arm of this `match` fits the value" at `gen.py --max`); that is
  fixed upstream, so nothing forces the packing now.
- **Goal it hurts:** readability; each fact once (the 18-bit width and the `p` offset are a second
  encoding of the link).
- **Ideal code:** `for l in flights.map(Flight).merge_sorted(portals.map(Portal), by: cost)`
  over two cost-sorted lists; `cpl run` of `fl.merge_sorted(po, by: cost)` over two `Link` enum
  lists prints them in cost order.
- **General fix:** none; an owed rewrite of the port.
- **Severity:** low

## Kruskal over a product graph is hand-rolled
- **Where:** the merged sort and two `Dsu`s.
- **What felt wrong:** `kruskal` takes one edge list; the product-graph variant (each kind weighted
  by the other side's component count) is written out.
- **Goal it hurts:** brevity.
- **Ideal code:** see the dream; std's `merge_sorted` of the two kinds by cost removes the packing.
- **General fix:** none needed: `merge_sorted` is the merged-by-key iterator.
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
