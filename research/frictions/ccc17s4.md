# Friction Report: ccc17s4

## Kruskal's tie order carries a fact silently
- **Where:** `let (_, take) = kruskal(pipes)`: old pipes win ties only because they come first
  and `kruskal` sorts stably; the header has to say so.
- **What felt wrong:** the requirement "old before new at equal cost" lives in input order.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** `kruskal(pipes, by: e => (e.w, not is_old(e)))` where the record knows its key.
- **General fix:** give `kruskal`'s `by:` the edge key too (`by: (e, r) => ...`), as `sorted(D, by:)`
  does for domain arrays.
- **Severity:** low

## Dream

```cp
let (n, m, d) = read<(1..=1e5, 1..=2e5, 0..=1e9)>()
domain House = [1..=n]
domain Pipe = [m]
let pipes = read<[Pipe](u: House, v: House, w: 1..=1e9)>()
let old = (p: Pipe) => p < Pipe[n - 1]
let tree = kruskal(pipes, by: p => (pipes[p].w, not old(p))).take
let dearest = max(pipes[p].w for p in tree)
let rest = Dsu.of(pipes[p] for p in tree if old(p) or pipes[p].w < dearest)
let saved = dearest? <= d and Pipe.any(p => p not in tree and old(p) and pipes[p].w <= d and rest.joins(pipes[p]))
print(Pipe.count(p => p in tree and not old(p)) - Int(saved))
```

## Measurements

- Tokens: port 284, original 786
- Max input (`gen.py --max`): port 0.03 s, 12484 KB, original 0.04 s, 7252 KB
- Emitted C++: 19578 bytes
