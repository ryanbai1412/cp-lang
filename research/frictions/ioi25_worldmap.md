# Friction report: ioi25_worldmap

## Dream

```cp
let (tree, tree_edge) = g.dfs_tree(root: V.first!)
let deep = tree.path(V.first!, V.max(by: v => tree.depth(v))!)
for v in tree.euler_tour(by: c => deep.contains(c)):  // the deep child last
    diags.push([v])
    if up[v].len > 0 and not expanded.contains(v):
        expanded.add(v)
        diags.extend([up[v], [v]])
```

Both programs paint one anti-diagonal of the 2n x 2n grid per Euler-tour visit of a DFS tree,
and give each vertex with back edges a `v | ancestors | v` triple. The dream is the judge's
`@grader fn create_map` and visits the deepest path last. The port is a stdin program that
reads the sample grader's input and gives each back edge to the end whose visit is nearer the
tour's middle. The dream as first written failed `cpl check` on the names below. Its current
form is what compiles. The quoted errors come from the first dream on a freshly built `cpl`.

## `V.max_by` is spelled `V.max(by:)`
- **Where:** `dream.cp`, `V.max_by(v => tree.depth[v])`, now `V.max(by: v => tree.depth(v))!`.
  `` error[E0203]: a `domain V` has no method `max_by` ``, with
  `help: write .max(by: ..): options are named arguments, so there is one max and no _by twin`.
- **What felt wrong:** `max_by` is the Rust and Kotlin spelling and the one a writer guesses.
  The diagnostic fixes it at once, so this costs one compile, not a lookup.
- **Goal it hurts:** coherence (guessability).
- **Ideal code:** either spelling. The design has chosen `max(by:)`, and the help line already
  routes `max_by` to it.
- **General fix (stdlib/diag):** none needed beyond the current help. Keep the help on every
  `_by` twin (`min_by`, `sort_by`, `max_by_key`).
- **Owner:** stdlib.
- **Severity:** low.

## `Bitset.insert` is spelled `add`
- **Where:** `dream.cp`, `if up[v].len > 0 and expanded.insert(v):`, now
  `not expanded.contains(v)` then `expanded.add(v)`. The repro
  `var s = Bitset<V>.empty(); if s.insert(V(1)): ...` fails with
  ``error[E0203]: Bitset<V> has no field or method `insert` ``, and the help lists `add`.
- **What felt wrong:** the dream wanted the test-and-set idiom ("true if newly added") from
  `HashSet::insert` and C++ `set::insert().second`. `Bitset.add` already returns that `Bool`
  (std/bitset.cp: `x and not y` in its example), but the help only lists method names. So the
  dream became two statements and looked up the same bit twice.
- **Goal it hurts:** express, coherence.
- **Ideal code:** `if up[v].len > 0 and expanded.add(v):`. That compiles today, once you know
  `add` returns whether it was new.
- **General fix (diag):** when an unknown method has a known alias on the receiver
  (`insert` -> `add`), name it in the help with its signature
  (`add(i) -> Bool: true if i was absent`), not only in the method list. This helps every set
  type (`Set`, `Bitset`, `DynBitset`).
- **Owner:** Consumers/Tools.
- **Severity:** low.
- **Status:** fixed by 59bed056c: `s.insert(v)` on a `Bitset` now offers `add`, which returns
  whether the bit was new.

## No way to orient a non-tree edge toward its ancestor
- **Where:** `dream.cp`,
  `adjacency(g.edges.filter(e => not tree.has(e)).map(e => tree.orient_down(e).rev))`, now
  a loop: `for e in g.edges().filter(e => not tree_edge[e.e]):` /
  `if tree.depth(e.u) > tree.depth(e.v): up[e.u].push(e.v) else: up[e.v].push(e.u)`.
  It fails with `error[E0203]: ... has no field or method has` and `... orient_down`. The
  first error is also because `dfs_tree` returns the pair `(tree, tree_edge)`.
- **What felt wrong:** in a DFS tree of an undirected graph, every non-tree edge joins a vertex
  to an ancestor. "Each back edge as (descendant, ancestor)" is a fact about the tree, but the
  port derives it again from depths, with a tie branch that can't happen. `Rooted.lower(u, v)`
  gives the child end only for tree edges and traps on a back edge.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** `let up = adjacency(g.edges().filter(e => not tree_edge[e.e]).map(e => tree.orient_down(e).rev))`,
  or `tree.back_edges()` yielding `(low: V, high: V)`.
- **General fix (stdlib):** let `Rooted.lower` (or a new `orient_down`) take any ancestor
  pair, which it can check with `is_anc`, or have `dfs_tree` also give its back edges already
  oriented. This helps every lowpoint, cycle-space and cactus port that walks back edges
  (bridges, 2-edge-connected parts, ear decompositions).
- **Owner:** stdlib.
- **Severity:** medium.

## `List.pad_to` is missing; `resized` copies
- **Where:** `dream.cp`, `diags.pad_to(4 * n - 1, with: diags.last!)`, now
  `while diags.len < 4 * n - 1: diags.push(diags.last!)`.
  `` error[E0203]: a `List<List<V>>` has no method `pad_to` ``.
- **What felt wrong:** std has `resized(n, fill)` (std/seq.cp), which builds a new list and also
  truncates. Nothing pads in place, so the dream used a `while` loop. The help line didn't
  point to `resized`.
- **Goal it hurts:** express, coherence.
- **Ideal code:** `diags.pad_to(4 * n - 1, with: diags.last!)`, or `diags = diags.resized(4 * n - 1, diags.last!)`
  if the copy is accepted.
- **General fix (stdlib):** a mutating `List.resize(n, fill)` next to `resized`, matching the
  `sort`/`sorted` pair. Give `pad`/`pad_to` misses a help line naming it. This helps any
  port that grows a table to a known length (DP rows, layered output).
- **Owner:** stdlib.
- **Severity:** low.

## A `@grader fn` can't return a fixed-size grid
- **Where:** `dream.cp`, the signature `-> [2 * n][2 * n]1..=n` and the body
  `return [2 * n][2 * n] (x, y) => ...`, now `-> List<List<1..=n>>` with
  `return map.map(row => row.collect()).collect()`. Restoring only the return type fails with
  ``error[E0501]: a `[Int][Int]Int` can't cross to the judge's grader in `create_map` `` and
  `help: a @grader fn takes and gives Int, F64, Bool, Char, Str, and lists and tuples of them`.
  The two-index table form `[a][b] (x, y) => e` also fails to parse:
  ``error[E0107]: expected the end of the statement, found `=>` ``, so the port nests
  `[2 * n] x => [2 * n] y => ...`.
- **What felt wrong:** the judge's type is `std::vector<std::vector<int>>`, and a sized array
  is that same shape with its length proven. The dream had to give up the size it knew, and
  then copy row by row to reach a `List`. The colours also had to become `Int`s (`.ord + 1`)
  before crossing, though `1..=n` is already an `Int` range.
- **Goal it hurts:** express, submission.
- **Ideal code:** `fn create_map(...) -> [2 * n][2 * n]1..=n:` returning the table directly.
- **General fix (compiler):** let `[n]T` and nested `[n][m]T` cross to the grader as
  `std::vector` (and nested vectors), with ranged ints crossing as `int`. Also accept `[a][b] (x, y) => e` as the 2-D table form. This helps
  every IOI task that returns a grid or matrix.
- **Owner:** Core.
- **Severity:** medium.

## A root `grader.cpp` is linked into the stdin port
- **Where:** the package layout. With `grader.cpp` and `worldmap.h` beside `port.cp`,
  `cpl test port.cp` fails with
  `cpl: /tmp/cpl-cache/port-<hash>.cpp has no fn the judge's grader .../grader.cpp supplies or calls`.
  The port is a stdin program, so `cpl stress` and the `--max` gate can run it alone. Moving
  the grader into `dream.tests/` links it only into the `@grader` dream.
- **What felt wrong:** `find_grader` (crates/cpl-contest/src/problem.rs) takes any `grader.cpp`
  next to a source, whether or not that source has a `@grader fn`. A package with a grader
  dream and a stdin port must hide the official grader in a folder named after one of its files.
- **Goal it hurts:** evidence, submission.
- **Ideal code:** `grader.cpp` + `worldmap.h` at the package root, linked into every program
  that declares a `@grader fn` and ignored by the others.
- **General fix (tools):** link a found `grader.cpp` only when the program has a `@grader fn`,
  or have `corpus.py` pass `--grader` for the dream only. This helps every IOI package that
  pairs a `@grader` dream with a stdin port.
- **Owner:** Consumers/Tools.
- **Severity:** medium.
- **Status:** fixed by 57c83b9a8: a found `grader.cpp` is linked only into a program that
  declares a `@grader fn`, so `grader.cpp` and `worldmap.h` now sit at the package root.

## The port and the dream build the map differently
- **Where:** `port.cp:23-33` against `dream.cp:14-26`.
- **What:** the dream tours the deepest root path last and expands each vertex at its first
  visit, then pads with `while diags.len < 4 * n - 1: diags.push(diags.last!)`. The port gives
  each back edge to the end whose roomiest visit is nearer the tour's middle and expands every
  vertex once, at that visit; its 2n - 1 visits plus n two-diagonal expansions make exactly
  4n - 1 diagonals, so it needs neither the padding nor the `expanded` set. The port came second
  and is the better rule; the dream keeps the deep-path order because it is the editorial's
  argument and both score in full on the package cases.
- **Cost:** none at runtime (n <= 40); two arguments to check for one problem.
- **General fix (idioms):** move the dream to the port's rule when the dream is next rewritten.
- **Severity:** low.

## Measurements

Contestant-written tokens (scripts/ports-page.py tokenizers): port 493, dream 425, original
1241 (1217 after `strip_cpp_library`; it includes the sample grader `main`). On the `gen.py
--max` input (50 scenarios, n = 40), the port takes 0.01 s and 3.5 MB, and `orig.cpp` takes
0.02 s and 5.1 MB, both under `-O2`. Both outputs pass `checker.py`. Emitted C++: 28201 bytes
(`cpl build`, judge `generic`).
