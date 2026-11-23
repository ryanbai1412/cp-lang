# egoi22p3 friction

The task is a grader function with judge-provided `GetMove`/`MakeMove`. `cpl run` has no stub
for a `@grader fn` (tools worklog, from apio23p1) and the corpus suite runs no interactor
(spec worklog, research/frictions/cco18p4.md), so spec.cp plays the game over stdin with a
`Wire` and the tests are transcripts of spec.cp against `interactor.py` on `tests/NN.case`.
`Graph.only(...)` not being a `Next` (stdlib, apio23p1) is why the per-component roots come
from a `Dsu` and not `components(rest)`.

## An inline comprehension argument to `Graph.undirected` mismatches its own key domain

- **Repro:** `fn f(n: Int, edges: List<(Int, Int)>): domain V = 1..=n; let g =
  Graph.undirected([edges.keys] e => (u: V(edges[e].0), v: V(edges[e].1)))` is E0302
  "expected `[keys_5](u: V_0, v: V_0)`, found `[keys_7](u: V_0, v: V_0)`"; binding the same
  comprehension to `let es` first and passing `es` checks and runs.
- **Goal it hurt:** express (the old spec.cp wrote it inline and stopped checking).
- **Routed:** compiler worklog.

## `g.next(u)` on a `Graph.undirected` is E0203

- **Repro:** `domain V = 1..=3; let es: [0..2](u: V, v: V) = [(u: 1, v: 2), (u: 1, v: 3)];
  print(Graph.undirected(es).next(V(1)).collect())` is E0203 "a `[V]List<V>` has no method
  `next` that `cpl run` knows", in `cpl check` too, though spec/lib/graphs-trees.md lists
  `g.next(u)` for every graph. spec.cp writes `g[home]`.
- **Goal it hurt:** express (one spelling for "where `u`'s arcs lead").
- **Routed:** compiler worklog.

## `Rooted.forest` wants exactly one root per tree, covering every vertex

- **Repro:** with `rest = g.only(v => v != home)`, `Rooted.forest(rest, from: List(g[home]))`
  traps `assertion failed: not seen[r]` (std/tree.cp:163) as soon as two friends of `home`
  share a component, and `home` itself is reached from no root.
- **Goal it hurt:** express. The natural "a DFS forest from the friends" needs a `Dsu` to keep
  one friend per component and `[home] +` so the isolated `home` is a tree of its own. `dfs`
  already skips a discovered source; `Rooted.forest` could do the same (skip a root already
  reached, leave unreached vertices out of every tree).
- **Routed:** stdlib worklog.
