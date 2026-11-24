# Friction Report: cco17p4

The v0.2 port ran the subset-sum DP over a `[Volume]Bool`, cloned it per pillar, tracked the
largest reachable volume `most` by hand, and rebuilt the tallest-first order with an index
comprehension. Now the reachable volumes are a `Bitset<Volume>` grown with one
`union_shifted(before, h - tall[i])` per distinct earlier height, the distinct heights are kept
as a list while placing, and the answer is `reach.each().collect()` as `List<Volume>`. The
`@alt` brute keeps its every-order count and the package gains tests.toml.

## Descending sort is `sorted().rev.collect()`
- **Where:** `let tall = height.sorted().rev.collect()`.
- **Ideal:** `height.sorted(desc: true)` or a `sorted_desc()`, without the extra copy.

## The `before` copy is spelled out
- **Where:** `let before = reach.clone()` so every shift reads the pre-pillar set.
- **Ideal:** `reach |= union(reach << (h - tall[i]) for h in levels)` with one temporary built
  by the library.

## Dream
```cp
let tall = height.sorted(desc: true)
var reach = Bitset<Volume>.single(Volume.first!)
for i in 1..n:
    reach |= union(reach << (h - tall[i]) for h in tall[1..i].distinct)
print(reach.each())
```

- Tokens: port 375, original 726
- Max input (`gen.py --max`): port 0.00 s, 3200 KB, original 0.45 s, 638464 KB
