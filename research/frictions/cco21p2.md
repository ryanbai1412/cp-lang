# Friction report: cco21p2

The port types the digits as `[d](-m..=m)` and replaces the old fixed 100-round loop with a
search that stops when 0 is reached or the frontier empties (the visited set makes it
finite), and prints with `none: "IMPOSSIBLE"`. The search itself stays hand-written:


## `bfs` over `Int` states does not check
- **Where:** the dream's `bfs(peel, from: [x], until: 0, hashed: true)` and `p.arcs(0)`; `spec.cp` hand-writes the level-by-level search with a `HashMap<Int, Int>` of back digits and walks it back by hand.
- **What felt wrong:** E0302 "`bfs` (`std.graph`) needs a domain for `V`, and `Int` is not one", with or without `hashed: true`. spec/lib/graphs-trees.md says storage follows the vertex type (any hashable `V` gives hash maps); std/graph.cp says `hashed: true` selects them and that an unbounded `V` without it is an error naming `hashed: true`. Neither runs: `bfs` builds dense `[V]?Int` tables. Repro: `fn halve(x: Int) = [x / 2]; print(bfs(halve, from: [10], until: 0)[0])`.
- **Goal it hurts:** express (a 15-line hand search for a one-line `bfs`), coherence (spec and std disagree on the spelling).
- **Ideal code:** `bfs(peel, from: [x], until: 0)` then `p.arcs(0)`.
- **General fix:** `Paths` tables follow `V` (dense over a domain, hash maps otherwise) for `bfs`, `bfs01`, `dijkstra`; settle on one spelling (spec's vertex-type rule or std's `hashed: true`) in spec and std. Helps every state-space search over unbounded states.
- **Severity:** medium.

## `x = 0` needs a nonempty path
- **Where:** `spell(0)` must find a cycle back to 0 (at least one digit); `bfs(from: [0], until: 0)` stops at once with `arcs(0) == []`.
- **What felt wrong:** even with hashed `bfs`, the search must start from the successors of `x`, not `x` itself.
- **Goal it hurts:** express (low).
- **Ideal code:** `bfs(peel, from: peel(x).map(.to), until: 0)` plus the first digit, or the hand search as today.
- **General fix:** none needed; noted so the rewrite to `bfs` keeps the edge case.
- **Severity:** low.

## Measurements
- Port (v0.3): `cpl stress` agrees with the old port and the accepted C++ (4543410) on 30 cases each; the widest max shape (gen.py 100 100000) takes 10.6 s against the C++'s 9.3 s (1.14x), the other three max shapes under 0.02 s, all with the C++'s output.
- Source bytes: spec.cp 1770, AC C++ 1928. Emitted C++ 16869 bytes.
- K = 2, D = 5001, Q = 5 at |n| = 1e18 (gen.py 100 100000, the widest search): spec native 11.0 s, AC C++ 9.45 s, same output; the official tests lack this shape (both exceed the limit). Other statement-max shapes (K = 1e6, D = K, missing residues): both under 0.02 s.
