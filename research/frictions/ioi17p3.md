# Friction report: ioi17p3

## Dream

```cp
let g = Graph<V>.read(tracks)
fn attract(target: Set<V>, player: Owner, on: Subgraph<V>) -> Set<V> = on.attractor(target, owner, player)
var alive = Set<V>.all()
loop:
    let good = attract(charging, Arezou, alive.induced(g))
    let bad = alive - good
    if bad.is_empty: break
    alive -= bad + attract(bad, Borzou, alive.induced(g))
print([Int(v in alive) for v in V])
```

## Game attractors are written by hand

- **Where:** `fn attract`.
- **What happened:** the attractor (backward BFS counting the opponent's remaining exits) is the
  core of every reachability game and std has no game-graph helper, so the port writes the queue,
  the exit counters and the `queued` marks itself.
- **General fix:** `attractor(target, controller, player)` over a graph view restricted to a
  vertex set.
- **Severity:** low.

## Vertex sets are `[V]Bool` tables

- **Where:** `alive`, `bad`, `good`, `lost`.
- **What happened:** set difference and union over a domain are spelled as per-vertex loops
  (`for v in V: if bad[v] or lost[v]: alive[v] = false`); no `[V]Bool` set algebra.
- **General fix:** a domain bitset with `-`, `+`, `any`.
- **Severity:** low.

## Measurements

Contestant-written tokens: port 406, original 407 (`corpus/audit.py` count). Emitted C++: 15174 bytes (`cpl build`, judge `generic`).
Max input (n = 5000, m = 20000) timed by `corpus/generators.py`.
