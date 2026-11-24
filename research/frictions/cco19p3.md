# Friction report: cco19p3

## Dream

```cp
let t = Rooted.from_parents(parent)          // parents given for 2..=n, the root has none
let below = t.up(+, finish: (s, v) => s + a[v])
let chain = t.down(root_chain, (up, v) => up + a[v] * (everyone - below[v]) - a[t.parent(v)!] * below[v])
```

## Hand tree walks become `Rooted`

The v0.2 port built `kids` lists, a hand BFS `order` and two loops over it for subtree people
and the rerooted `chain`. The rewrite takes `Rooted.from_parents`, `up` for the people under
each city and `down` for `chain`; `children`/`parent` give each hub's branches. The subset
sums come from std `subset_sums` (already sorted), and the hand two-pointer sweep is a
`last_lt` per right-half sum.

## Remaining friction

- `from_parents` wants a `[V]?V`, so the `[Kid]V` read is widened with
  `Kid.at(Int(v)).map(...)`. A `from_parents` over a sub-domain of `V` would take it as read.
- `down`'s step sees only the child, so the parent comes back through `t.parent(v)` with a
  `None` arm that only the root reaches.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. Corpus tests pass.
