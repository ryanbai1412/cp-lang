# Friction Report: ccc10s4

## The outside is a sentinel region
- **Where:** `domain Region = 0..=pens` / `let outside = Region.first!`.
- **What felt wrong:** pens are `1..=pens`; the extra vertex is a numbering trick, and
  `for pen in Region.after(outside)` re-states it.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** `domain Pen = 1..=pens` / `domain Region = Pen + {outside}` (a domain extended by
  named points), with fences `(u: Region, v: Region)`.
- **General fix:** domain sums with named extra points (source/sink in flows, outside in planar
  duals, a virtual root in forests) as one core form.
- **Severity:** low

## `zip` is free, `cyclic_pairs` is a method
- **Where:** `for ((a, b), cost) in zip(posts.cyclic_pairs(), costs):`.
- **What felt wrong:** `posts.cyclic_pairs().zip(costs)` is the guess; it fails with `error[E0203]: a
  List<(Int, Int)> has no method zip` ("`zip` is a free function").
- **Goal it hurts:** coherence.
- **Ideal code:** `posts.cyclic_pairs().zip(costs)`.
- **General fix:** one spelling for sequence combinators: methods, with the free form as the alias.
- **Severity:** low

## `kruskal` cannot say whether the forest spans
- **Where:** `if take.count(t => t) == regions - 1: total else: None` after `kruskal(...)`.
- **What felt wrong:** every caller that needs a spanning tree re-derives "it spans" from the
  taken-edge count and the vertex count. `cpl run` of `kruskal(es, spanning: true)` fails with
  `error[E0310]: kruskal has no parameter named spanning` ("its parameters are `edges`, `by`").
- **Goal it hurts:** obvious way is correct (the count check is easy to get off by one or forget).
- **Ideal code:** `kruskal(fences, spanning: true)?.total`.
- **General fix:** a `spanning:` flag (or a `spans` field in the result) on `std.graph.kruskal`,
  returning `None` when the edges leave more than one component.
- **Severity:** low
- **Owner:** stdlib.

## Dream

```cp
let pens = read<1..=100>()
domain Pen = [1..=pens]
domain Region = Pen + {outside}
let sides = Pen.flat_map(pen => let m = read<3..=8>(); zip(read<[m]1..=1000>().cyclic_pairs(), read<[m]1..=5000>())
    .map(((a, b), w) => (side: sorted(a, b), pen:, w:))).group(by: s => s.side)
let fences = [(u: s[0].pen, v: s.get(1)?.pen ?? outside, w: s[0].w) for s in sides.values]
let tree = (keep: Region -> Bool) => kruskal(fences.filter(f => keep(f.u) and keep(f.v)), spanning: true)?.total
print(min(tree(r => r != outside), tree(_ => true))!)
```

## Measurements

- Tokens: port 321, original 852
- Max input (`gen.py --max`): port 0.00 s, 3328 KB, original 0.00 s, 3584 KB
- Emitted C++: 24319 bytes
