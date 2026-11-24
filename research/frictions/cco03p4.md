# Friction Report: cco03p4

The v0.2 port wrapped everything in `main_solve()` for a local domain and kept Int bitmasks with a
`bit(k)` helper and `Mask(s)` casts. It now reads `rules: [k](x: V, y: V)` directly, keeps
`need: [V]Subset<V>`, and fills `ways: [Subset<V>]Int` with `with`/`contains`/`-`. Adds gen.py and
tests.toml.

## "need is inside s" is spelled as a difference (not a gap: `<=` exists)
- **Where:** `(need[v] - s).is_empty`.
- **Ideal:** `need[v] <= s`, which `Subset<B>` already has: `cpl run` of
  `let a = Subset<V>.empty.with(V(1)); let b = a.with(V(2)); print(yes(a <= b))` prints `YES`.
  An owed rewrite of the port, not a std gap.

## The count of rules has no stated bound
- **Where:** `let k: Int = read()`; the statement as kept in the corpus gives no limit for k.

## Dream
```cp
let n: 1..=9 = read()
let k: Int = read()
domain V = 1..=n
let rules: [k](x: V, y: V) = read()
let need = [V] y => Subset(r.x for r in rules if r.y == y and r.x != y)
var ways: [Subset<V>]Int = 0
ways[.empty] = 1
for s in Subset<V>:
    for v in s.complement.items if need[v] <= s: ways[s.with(v)] += ways[s]
print(ways[.full])
```

- Tokens: port 169, original 356
- Max input (`gen.py --max`): port 0.00 s, 3200 KB, original 0.00 s, 3456 KB
