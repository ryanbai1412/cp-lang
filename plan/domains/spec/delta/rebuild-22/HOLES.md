# Rebuild round 22: holes

Cold read of `spec/` and root `std/` only, at `45aa99f08`. Ten Codeforces
problems from contests 1981-2010, rated 1700-2300, none already under
`rebuild-*/`. Written before any `cpl` or `cargo` command ran.

## Problems

| File | Problem | Rating | Covers |
|---|---|---|---|
| `01-1988D.cp` | The Omnipotent Monster Killer | 2000 | tree DP over `t.order.rev`; 64-bit (sum up to 6e18) |
| `02-2006B.cp` | Iris and the Tree | 1800 | tree, Euler (DFS) numbering: subtree = label range |
| `03-2000G.cp` | Call During the Journey | 2100 | shortest path (latest-departure Dijkstra) |
| `04-1998D.cp` | Determine Winning Islands in Race | 2100 | BFS; 1-based islands, 0-based answer string |
| `05-2001D.cp` | Longest Max Min Subsequence | 1900 | sorted set (`SortedSet`) + heaps |
| `06-2009G1.cp` | Yunli's Subarray Queries (easy) | 1900 | sliding window, `Counter` |
| `07-1995B2.cp` | Bouquet (Hard Version) | 1700 | 64-bit overflow risk (m, c a up to 1e18) |
| `08-1994E.cp` | Wooden Game | 2000 | bitwise OR greedy |
| `09-1991D.cp` | Prime XOR Coloring | 1900 | XOR, constructive any-answer, `@check` |
| `10-1986F.cp` | Non-academic Problem | 1900 | bridges, bridge tree, tree DP |

## Summary

| Class | Count |
|---|---|
| silent | 12 |
| contradiction | 2 |
| missing-stdlib | 1 |
| found-late | 0 (before running) |
| **total** | **15** |

Every hole is inside the Milestone 1 subset (`spec/SUBSET.md`): each lies in
a section or library marked supported (R04.2, R04.4, R06.4, R07.7, R10.2,
R11.3, std-graph, std-tree, std-heap, std-bag, std-hashmap, std-math).
H04 is already listed there as a known gap
(`core-11/prelude-has-components.cp`).

## Top five risks

1. **H03** time-dependent shortest paths have no library form: `03` and its
   revision hand-roll Dijkstra (heap, stale check, relax) because an arc's
   cost depends on the arrival time.
2. **H01** `Rooted.from_parents` takes `[V]?V`, but its doc says the input
   shape `read<[V.drop(1)]V>()` goes straight in; if not, every
   "parents of 2..n" problem needs a conversion line.
3. **H05** comparing a position with an `Int` (`next <= r`): arithmetic on
   positions gives `Int`, but nothing says whether `<` mixes them. Wrapped
   in `Int(...)` everywhere; noisy if unnecessary, a trap if guessed wrong.
4. **H08** `heap.drain().find(p)`: whether breaking out of `drain` early
   leaves the rest in the heap. `05` depends on it for lazy deletion.
5. **H09** `condense(g, comp)` on an undirected graph: whether it emits
   both directions per bridge so `Rooted` can root it. `10` depends on it.

## Holes

| ID | Class | Where | What I guessed | Files |
|---|---|---|---|---|
| H01 | silent | `std/tree.cp` `Rooted.from_parents(par: [V]?V)` vs its doc ("parents of 2..n read as `read<[V.drop(1)]V>()`") | the `[V.drop(1)]V` table is accepted directly, the missing key being the root | 02 |
| H02 | contradiction | `spec/lib/sequence-patterns.md` Track table names `Bag<T>` with `b.max!`, `b.min!`; `std/bag.cp` has only `SortedBag` with `.first`/`.last` | `SortedBag`/`SortedSet` with `.first` | 05 |
| H03 | missing-stdlib | `spec/lib/graphs-trees.md` shortest paths (static `.w` only; Open 1 "weight algebra"); `std/graph.cp` `dijkstra` | hand-rolled Dijkstra with `Heap(by: desc)` | 03, 03-rev |
| H04 | contradiction | `spec/core/11-modules-prelude.md` (chapter-table names need no import) vs `std/prelude.cp` (no `components`, `bridges`, `two_edge_components`) | `use std.graph.{two_edge_components}` | 10 |
| H05 | silent | `spec/core/04-indices.md` position arithmetic gives `Int`; comparison of a position with an `Int` variable unstated | not allowed; wrote `Int(p)` | 05, 06 |
| H06 | silent | `spec/core/04-indices.md`: is `^` (and `%`) "arithmetic" on positions? | `v % 4` works on a position; `^` wrapped as `Int(u) ^ Int(v)` | 09 |
| H07 | silent | `spec/lib/random-stress.md` `read_output()`: which types it reads (counted domain arrays, range with a runtime bound read from the output) and whether a `domain` may be declared inside the `@check` fn | `read_output<[V]1..=k>()` with `domain V` declared in the fn | 09 |
| H08 | silent | `std/heap.cp` `drain(mut self) -> yields T` | pops one element per yield; leaving early keeps the rest | 05 |
| H09 | silent | `spec/lib/graphs-trees.md` `condense(g, comp)` on an undirected graph | both directions, one arc per bridge; `Rooted` accepts `[C]List<(to: C)>` | 10 |
| H10 | silent | `spec/core/04-indices.md`: is a domain itself a sequence with `.min(by:)`, `.filter`, `.map`, `.any`? | yes (`Round.min(by:)` is the argmin position) | 01, 09, 10 |
| H11 | silent | `spec/core/01-lexical-grammar.md` `for x in xs if p`: is `p` evaluated per element after earlier bodies mutated its captures? | yes, per element, lazily | 03-rev |
| H12 | silent | `spec/core/02-values-types.md`/`06`: tuple pattern in `while let (a, b) = h.pop()` | allowed | 03, 03-rev |
| H13 | silent | prelude `min`/`max`: arity beyond two not stated | nested two-argument calls | 07 |
| H14 | silent | `Counter<Int>(0)` in `spec/lib/sequence-patterns.md` vs `Counter.new(init)` in `std/hashmap.cp`: is `T(args)` sugar for `T.new(args)`? | yes | 06 |
| H15 | silent | `spec/core/10-io.md`: no way to skip a counted run of tokens | `let _parents = read<[n - 1]1..=n>()` | 08 |

## Per-problem mapping

| File | Holes |
|---|---|
| 01-1988D | H10 |
| 02-2006B | H01 |
| 03-2000G | H03, H12 |
| 04-1998D | none |
| 05-2001D | H02, H05, H08 |
| 06-2009G1 | H05, H14 |
| 07-1995B2 | H13 |
| 08-1994E | H15 |
| 09-1991D | H06, H07, H10 |
| 10-1986F | H04, H09, H10 |
| 01-1988D-rev | H10 |
| 03-2000G-rev | H03, H11, H12 |

## Revision test

| File | Change | Lines / chunks changed | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|
| `01-1988D-rev.cp` (tree) | answer per vertex: least damage if only `v`'s subtree existed | 1 line, 1 chunk (the final `print`: `Round.map(...)` over `V.first!` becomes `V.map(v => ...)`) | none | none | none |
| `03-2000G-rev.cp` (other) | several non-overlapping calls `(t1, t2)` instead of one | 3 chunks: input (1 line becomes 2), a new 5-line `depart` fn, the bus line (2 lines become 1) | whether the `for ... if` filter sees `dep` after the body changed it (H11); whether `.sorted(by:)` on a counted array of records is the right materialization | none | none |

## After running

Built with `cargo build -p cpl-cli` at `45aa99f08`. Each file went through
`cpl check`, `cpl run` on every sample, `cpl test`, `cpl build`,
`g++ -std=c++17 -O2` and the native binary on every sample. The harness is in
`stress/`: `stress.py` (generator + brute per problem, plus a validator for
`1991D`), `run.sh FILE ID N` (N random inputs, comparing `cpl run`, the native
build and the brute) and `maxgen.py` (one maximum-size input per problem).
The two revisions have no official samples: `01-1988D-rev` ran on the
`01` sample, `03-2000G-rev` on the hand-written `stress/rev-03-2000G.in`;
both agree between `cpl run` and native.

### Per-file status (final source)

| File | check | run (samples) | test | build + g++ | native (samples) | stress (200 random) | native, max input |
|---|---|---|---|---|---|---|---|
| 01-1988D | ok | ok | ok | ok | ok | 0 mismatches | 0.18 s |
| 02-2006B | ok | ok | ok | ok | ok | 0 mismatches | 0.07 s |
| 03-2000G | ok | ok | ok | ok | ok | not stressed | 0.03 s |
| 04-1998D | ok | ok | ok | ok | ok | not stressed | 0.05 s |
| 05-2001D | ok | ok | ok | ok | ok | 0 (2 before the B2 workaround) | 0.17 s |
| 06-2009G1 | ok | ok | ok | ok | ok | 0 mismatches | 0.03 s |
| 07-1995B2 | ok | ok | ok | ok | ok | 0 mismatches | 0.05 s |
| 08-1994E | ok | ok | ok | ok | ok | 0 mismatches | 0.02 s |
| 09-1991D | **FAIL (B1)** | valid answer, differs from the sample text (any answer allowed) | **FAIL (B1)** | ok | valid, same as `cpl run` | 0 invalid (Python validator: valid and minimum colours) | 0.01 s |
| 10-1986F | ok | ok | ok | ok | ok | 0 mismatches | 0.04 s |
| 01-1988D-rev | ok | runs (agrees with native) | n/a | ok | n/a | not stressed | not timed |
| 03-2000G-rev | ok | runs (agrees with native) | n/a | ok | n/a | not stressed | not timed |

Eight problems were stressed (1600 random inputs). No build is over 1 s on
its maximum input (largest 0.18 s). Every sample also ran clean under
`-fsanitize=address,undefined`; the one crash (05 before the workaround)
reproduced under the sanitizers as the language's own `!` on `None` trap,
with no sanitizer report. The generated C++ for `05` (24.9 KB) and `10`
(25.3 KB) is over the 24 KB size warning, under the 32 KB target.

### Failures

Every edit made after running, in order. The `.cp` files hold the final
(passing) form.

| # | File | Symptom | Class | Smallest reproduction | Edit |
|---|---|---|---|---|---|
| F1 | 01, 01-rev | E0309 `*` can't be applied to `Round` and `Int` | spec hole (H16) | `domain V = 1..=4` / `print(V.first! * 2)` | `Int(r) * a[v]` |
| F2 | 02 | E0504 "not implemented yet: a fresh type made from values, `type Z = Mint(mod: p)`" on `read<[V.drop(1)]V>()` | implementation bug (B3) | `let n: 2..=5 = read()` / `domain V = 1..=n` / `let p = read<[V.drop(1)]V>()` | read parents in a loop into `var par: [V]?V = None` |
| F3 | 03, 03-rev | E0302 expected `Desc<Int>`, found `Desc<(Int, V)>` | implementation bug (B4) | `var h: Heap<(Int, Int)> = Heap(by: desc)` | `MaxHeap<(Int, V)> = MaxHeap()` |
| F4 | 05 | E0302 expected `Int`, found `(Desc<Int>, P)` | implementation bug (B4) | `var h: Heap<(Int, Int)> = Heap(by: e => (desc(e.0), e.1))` | store `(-value, position)` in a min-heap |
| F5 | 06 | E0309 `-` can't be applied to `Int` and `P` | spec hole (H16) | `domain V = 1..=4` / `print(5 - V.first!)` | `a[i] - Int(i)` |
| F6 | 09 | E0201 no `is_prime` in scope (help names the import) | reader error (H17) | `print(yes(is_prime(7)))` | `use std.math.{is_prime}` |
| F7 | 09 | E0309 `%` can't be applied to `V` and `Int` | spec hole (H06) | `domain V = 1..=4` / `print(V.first! % 2)` | `Int(v) % 4` |
| F8 | 09 | E0501 "`read_output` is not in scope during lowering" in `cpl check`, `cpl test` and `cpl judge`; also fails on `spec/conformance/core-06/check-read-output-type-argument.cp` | implementation bug (B1) | `@check` / `fn ok() -> Bool:` / `    let n: Int = read()` / `    let out = read_output<Int>()` / `    out == n` | none possible; 09 stays FAIL on check/test |
| F9 | 10 | E0302 `two_edge_components` needs a domain for `V`, and `Int` is not one | implementation bug (B6) | `domain V = 1..=n` / `let g = Graph.undirected(read<[m](u: V, v: V)>())` / `two_edge_components(g)` (`bfs(g, ...)` on the same `g` is fine) | name the edge domain: `domain E = 0..m`, `read<[E](u: V, v: V)>()` |
| F10 | 05 | stress: 2/200 traps `!` on `None` (seeds 81, 150; `n = 8`, `a = 6 7 8 1 3 3 8 1`), interpreter and native alike | implementation bug (B2) | `var h: Heap<Int> = Heap()`; push 3, 1, 2; `let x = h.drain().find(v => v == 1)!`; `print(x, h.len)` prints `1 0`, doc says `1 2`; same with `for v in h.drain(): if v == 1: break` | `while let e = h.pop(): if fresh: ... break` |
| F11 | 05 | after F10: E0302 "lambda parameter has type `(Int, P)`, but `(_, _)` is expected" | implementation bug (B5) | `let f = (e: (Int, Int)) => e.0 + e.1` / `print(f((1, 2)))` (passing a tuple variable works) | two-parameter lambda `(x: Int, p: P) => ...` |

### Implementation bugs

| ID | Bug | Where it bites |
|---|---|---|
| B1 | `read_output` is rejected in every `@check` (E0501 during lowering), so no checker can run; the core-06 conformance file fails the same way | 09 |
| B2 | `Heap.drain()` left early pops every remaining element, in the interpreter and the C++; `std/heap.cp` documents the opposite | 05 (wrong answer, not a compile error) |
| B3 | `read<[V.drop(1)]V>()`, the shape `Rooted.from_parents`' doc names, fails with an unrelated "fresh type made from values / Mint" diagnostic | 02 |
| B4 | `Heap(by:)` infers the key as `Int`: `by: desc` and tuple keys holding `Desc` fail on tuple elements | 03, 03-rev, 05 |
| B5 | a tuple literal passed to a lambda whose single parameter is a tuple does not type-check | 05 |
| B6 | a graph built from `read<[m](u: V, v: V)>()` (anonymous count) loses `V` for `two_edge_components`, while `bfs` accepts it | 10 |

### Holes after running

| ID | Outcome |
|---|---|
| H01 | guess wrong in practice: the documented read shape does not compile (B3) |
| H02 | `SortedSet`/`SortedBag` with `.first` worked |
| H03 | confirmed: hand-rolled Dijkstra works, still no library form |
| H04 | confirmed: `two_edge_components` needs the import |
| H05 | guess right: `V < Int` is E0302 "can't compare a `V` with a `Int`" |
| H06 | guess wrong: `V % Int` and `V ^ Int` are both rejected |
| H07 | blocked by B1, unanswered |
| H08 | guess matches the doc; the implementation disagrees (B2) |
| H09 | guess right: `Rooted(condense(g, comp))` roots the bridge tree (0/200) |
| H10 | guess right |
| H11 | not exercised beyond the hand input |
| H12 | guess right |
| H13 | `max(1, 5, 3)` works: variadic |
| H14 | guess right |
| H15 | guess right |
| H16 | **found-late**, silent. `spec/core/04-indices.md` (R04.3.3) "Arithmetic on a position gives an `Int` (`v + 1`)", but `*`, `%`, `^` and `Int - V` are rejected; only `V ± Int` works. Either the spec should list the operators or the compiler should accept them |
| H17 | **found-late**, silent. `spec/core/11-modules-prelude.md` "a chapter's main tables" decide the prelude, but `spec/lib/math.md` lists `is_prime` only in a signature block, so it is not prelude; nothing marks which listing counts |

Final counts: 17 holes (12 silent, 2 contradiction, 1 missing-stdlib,
2 found-late), all inside the Milestone 1 subset; 11 failures after running
(1 reader error, 3 spec hole, 7 implementation bug) from 6 distinct
implementation bugs.
