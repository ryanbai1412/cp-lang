# Friction Report: ccc22s5

The v0.2 `opt_add` / `opt_min` helpers are gone. Lifted `+`/`-` and `min`/`min=` (None as identity)
on `?Int` say "skip impossible states" directly. The bounds are types (`2 <= n`, `cost <= 1000`),
the Y/N string is read as `[V]Char`, and the package gains gen.py and tests.toml.

## `+=` doesn't lift to `?Int`
- **Where:** `settled = settled + min(done[c], push[c])`.
- **What felt wrong:** `+` lifts, but `+=` is rejected (E0309), so a running sum of optional costs
  repeats the target.
- **Ideal:** `settled += x` with the lifted meaning (None stays None), the same as `x = x + y`.

## Three-way `min` nests
- **Where:** `min(need[c], min(done[c], push[c]))`.
- **Ideal:** variadic `min(need[c], done[c], push[c])`.

## Dream
```cp
let n: 2..=2e5 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let initial: [V]Char = read()
let cost: [V]1..=1000 = read()
let tree = Rooted(Graph.undirected(edges), V.first!)
var (push, done, need): ([V]?Int, [V]?Int, [V]?Int) = (None, None, None)
for u in tree.at.rev:
    let kids = tree.children(u)
    let settled = kids.sum(c => min(done[c], push[c]))
    let helped = kids.sum(c => min(need[c], done[c], push[c]))
    let y = initial[u] == 'Y'
    let lit_push = if y: helped else: kids.min(c => helped - min(need[c], done[c], push[c]) + push[c])
    let lit_done = if y: settled else: kids.min(c => settled - min(done[c], push[c]) + push[c])
    push[u] = cost[u] + lit_push
    done[u] = min(lit_done, push[u])
    need[u] = min(done[u], settled, cost[u] + helped)
print(min(done[V.first!], push[V.first!])!)
```

- Tokens: port 372, original 797
- Max input (`gen.py --max`): port 0.06 s, 23404 KB, original 0.09 s, 22912 KB
