# Rebuild round 19: holes

Cold read of `spec/` and root `std/` only, then ten Codeforces problems from contests
1922-1933 (rated 1600-2100, none used by earlier rounds). Everything above
`## After running` was written before any build or run.

## Summary

| Class | Count | In Milestone 1 subset |
|---|---|---|
| silent | 11 | 11 |
| contradiction | 1 | 1 |
| missing-stdlib | 3 | 3 |
| found-late | 0 | 0 |
| **total** | **15** | **15** |

"In subset" means the rule section or library family is *supported* in `spec/SUBSET.md`
(core R01-R10, std-graph, std-tree, std-dsu, std-mod, std-math, std-seq, std-strings, std-io).

## Top five risks

1. **H01** nested counted-array read `read<[k][n]V>()` / `read<[n][m]0..=1>()` (01, 08): if it is
   not row-per-line or not accepted at all, two solutions fail at the first read.
2. **H03** the graph helpers accept a plain `[V]List<V>` adjacency (`topo`, `bfs`), since
   `Graph.undirected` cannot take a filtered edge subset (01, 07).
3. **H05** a top-level lambda capturing a top-level `Comb` used from the per-test section (02).
4. **H08** `match` on a std enum with unqualified arms and `()` as an empty arm (04).
5. **H12** `Paths.path(t)` order (source first) and `until:` taking a vertex (07); a reversed
   path still prints a valid cycle, but a wrong `until` type fails to check.

## Holes

| ID | Class | Location | Question | What I guessed | M1 |
|---|---|---|---|---|---|
| H01 | silent | `core/10-io.md` 10.2 (reads); only `[R, C]Char` grids are shown | How is `[k][n]T` (array of counted arrays) read, and is a `[n][m]` of ints read row by row? | Tokens in row-major order; `x[i][j]` indexes it | yes (R10.2) |
| H02 | silent | `core/02` 2.8, `core/07` 7.8 | Is `[V] _ => []` the way to fill a `[V]List<V>`, with the element type taken from the declaration? | Yes | yes (R02.8) |
| H03 | missing-stdlib | `lib/graphs-trees.md` Graph construction; `std/graph.cp` `Graph.undirected(edges: [E]R, keep:)` | Building a graph from a *subset* of the edges (`es.filter(...)`) is not offered: `keep` maps arcs, it cannot drop them, and a filter view is not a `[E]R` | Hand-built `[V]List<V>` adjacency, passed to `bfs` / `topo` as a `Next<V>` (std's `dfs` doc example uses `[V]List<V>`) | yes (std-graph) |
| H04 | silent | `core/06` 6.2 lambdas | May a lambda annotate its parameter types, `(s: Int, y: Int) => ...`? | Yes | yes (R06.2) |
| H05 | silent | `core/10` 10.1 `@!multitest`, `core/06` 6.2 | Can the per-test section call a lambda defined in the shared section that captures a shared `Comb`? | Yes; shared bindings are visible below the divider | yes (R10.1) |
| H06 | silent | `core/10` 10.2 | Does `let (a, b, c, d) = read<(R1, R2, R3, R4)>()` destructure a tuple read with range-typed parts? | Yes | yes (R10.2) |
| H07 | contradiction | `lib/graphs-trees.md` tree table: `t.parent(v)`; `std/tree.cp` also exposes `pub par: [V]?V`; `t.tour` yields `Visit<V, V>` with `via` = parent while `dfs` yields `Visit<V, Arc>` with `via` = arc | Same enum, two meanings of `via`; and two spellings for the parent (`t.parent(v)` vs `t.par[v]`) | `t.tour`, reading `via` as the parent vertex | yes (std-tree) |
| H08 | silent | `core/01` 1.7 `match` | Are arms of a std enum (`Visit`) written unqualified (`Enter(v, p) =>`), must every variant be covered, and what is the empty arm body? No `pass` in the grammar | Unqualified arms, `Again(_, _) => ()` | yes (R01.7) |
| H09 | silent | `core/01` 1.6 `if let` with comma conditions | `if let p = x, alive[p]:` (a binder followed by a plain condition) | Allowed; the condition sees `p` | yes (R01.6) |
| H10 | silent | `core/07` 7.11 `sorted(by:, desc:)`; `std/algebra.cp` `desc(x)` | Two spellings of descending order | `sorted(by: key, desc: true)` | yes (R07.11) |
| H11 | silent | `std/seq.cp` `adjacent` on a slice `row[1..]` of a counted array | Does `.adjacent` on a slice yield element pairs (values, not keys)? | Yes, `(T, T)` values | yes (R07.4) |
| H12 | silent | `std/graph.cp` `Paths.path(t) -> ?List<V>`, `bfs(until:)` | Path order and whether `until` takes a plain `V` | Source first, `until: V` | yes (std-graph) |
| H13 | missing-stdlib | `lib/graphs-trees.md` traversal; `bfs` needs a `Next<V>` | A BFS over an implicit state space (cyclic grid with custom moves) has no closure-based form; implementing `Next<V>` for a user type is not shown | Hand-written queue over flat `Int` states | yes (std-graph) |
| H14 | missing-stdlib | `lib/containers.md` / `std/interval.cp` not covering "count of segments covering each point" | No difference-array helper surfaced from the spec tables | `[0] * (n + 2)` with `+1/-1` and a running sum | yes (std-prefix) |
| H15 | silent | `core/07` 7.8 comprehensions, `core/02` 2.8 | `[v for v in V]` makes a `List<V>`, and `cand = []` re-infers the element type from the `var` | Yes | yes (R07.8) |

## Per-problem mapping

| # | Problem | Topic | Std used | Holes |
|---|---|---|---|---|
| 01 | 1931F Chat Screenshots | toposort | `topo`, `.adjacent`, `yes` | H01 H02 H03 H11 |
| 02 | 1931G One-Dimensional Puzzle | combinatorics, mod inverse | `Mod`, `Comb.choose` | H04 H05 H06 |
| 03 | 1929D Sasha and a Walk in the City | tree DP, multitest | `Graph.undirected`, `Rooted`, `order`, `children`, `.sum()` on `Mod` | - |
| 04 | 1923E Count Paths | trees, DFS events | `Rooted`, `t.tour`, `Visit` | H07 H08 |
| 05 | 1932F Feed Cats | interval DP + sweep | `min=`, domain positions | H14 |
| 06 | 1928C Physical Education Lesson | number theory (divisors) | `divisors`, `.count` | H04 H06 |
| 07 | 1927F Microcycle | DSU + BFS path, sorting | `Dsu`, `sorted(by:, desc:)`, `bfs`, `Paths.path` | H02 H03 H10 H12 |
| 08 | 1933F Turtle Mission | grid shortest path (BFS) | `min=` on `?Int` | H01 H13 |
| 09 | 1930D1 Sum over all Substrings | strings, greedy | `Str(len:)` read | - |
| 10 | 1922D Berserk Monsters | linked list sweep, large multitest | `v.prev/next`, `?.map`, `??` | H09 H15 |
