# Friction Report: dmopc20c7p5

## 1. Sum domains (`domain Node = Item + Pair`)

- **Area:** core
- **Goal:** express
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_15/dream/dmopc20c7p5.cp:14-18`
- **Port:** `proto/ports/scout_batch_15/dmopc20c7p5.cp:19-25`

Project selection has two distinct node kinds (`Item` and `Pair`). Without sum domains `Item + Pair`, the port flattens both into `domain Node = 0..n + m` with manual `item_node`/`pair_node` offset helpers (`i.ord`, `n + p.ord`).

## 2. `max_closure` requires a domain-indexed `[E](u: V, v: V)` edge table instead of `List<(u: V, v: V)>`

- **Area:** flows
- **Goal:** express
- **Severity:** low
- **Dream:** `proto/ports/scout_batch_15/dream/dmopc20c7p5.cp:18`
- **Port:** `proto/ports/scout_batch_15/dmopc20c7p5.cp:27-31`

Because `max_closure` takes `[E](u: V, v: V)`, the port introduces a synthetic `domain Need = 0..2 * m` and decodes `(k.ord / 2, k.ord % 2)` instead of passing a comprehension list `[(u: Node(p), v: Node(i)) for p in Pair for i in [pairs[p].u, pairs[p].v]]`.
