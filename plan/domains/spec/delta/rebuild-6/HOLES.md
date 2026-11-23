# Rebuild round 6: holes

I read commit 136ca9833 and only `spec/` (README.md, core/, lib/, tools/, diagnostics.md, GLOSSARY.md; not delta/ or conformance/) and the stdlib stubs in `std/`. I did not run the compiler. Every place below is one where I had to guess while writing the ten solutions.

## Summary

There are 26 holes: 22 silent, 3 contradiction, 1 missing-stdlib.

The ten most likely to hit a contestant, most likely first:

1. H-FW-WEIGHT-WIDTH
2. H-SEGTREE-NO-OP
3. H-RANGEFENWICK-GONE
4. H-MOD-FILL
5. H-STR-FROM-CHARS
6. H-DIR-FROM-STEP
7. H-FW-RESULT-DROP
8. H-TAG-RANGE-READ
9. H-TREE-EDGES
10. H-GRID-FIND

## Silent

### H-FW-WEIGHT-WIDTH
- Question: `g.matrix()` gives `[V, V]?W`, where `W` is the arc weight type. When weights are read as `w: 1..=1e9`, is `W` the range `1..=1e9`, so that path sums of several roads overflow the range when `floyd_warshall` stores them?
- Readings: (a) `W` stays the declared range, and storing a sum above 1e9 traps; (b) the matrix widens to `Int`; (c) ranges only constrain input, and the stored type is `Int`.
- Chose: (a). I pass `keep: r => (to: r.to, w: Int(r.w))` so that `W = Int` (04-shortest-routes-ii.cp).
- Cost if wrong: under (a), the natural `Graph.undirected(read<...w: 1..=1e9>())` followed by `matrix()` traps on most tests. Under (b) or (c), the `keep:` is only noise.

### H-SEGTREE-NO-OP
- Question: the ops-ranges table lists "range apply, point read" as `SegTree(a: a, act:)` with no fold op. What does `st[k]` return without an op, and does it include every `apply` that covers `k`?
- Readings: (a) `st[k]` is the leaf with all pending tags applied; (b) a point read needs an op such as `SegTree(+, a, act: add)` and `st.fold(k..=k)`.
- Chose: (a), with `add` from std/algebra.cp:551 (05-range-update-queries.cp).
- Cost if wrong: a compile error, or stale values if the tags are not pushed into the leaf.

### H-MOD-FILL
- Question: can an integer literal fill and assign `Mod<P>` slots, as in `var ways: [0..=x]Z = 0` and `ways[0] = 1`?
- Readings: (a) integer literals convert to `Mod<P>` where a `Mod` is expected; (b) you must write `Z(0)` / `Z(1)`.
- Chose: (a) (02-coin-combinations-i.cp).
- Cost if wrong: a compile error at the two lines every counting DP opens with.

### H-STR-FROM-CHARS
- Question: how do you turn a generator of `Char` into one printed word?
- Readings: (a) `"".join(gen)` (std/strings.cp:296 takes a `Seq`); (b) `Str(gen)`; (c) `print(gen)` with an empty separator option.
- Chose: (a) (01-labyrinth.cp).
- Cost if wrong: a compile error, or the moves printed separated by spaces (wrong answer).

### H-FW-RESULT-DROP
- Question: `floyd_warshall` returns a `Bool`. R06.4.8 says dropping it triggers the discarded-result lint, and that `let _ = f()` drops it too. What is the clean way to ignore it, and is the call inside `assert` still evaluated in release builds?
- Readings: (a) `let ok = ...; assert ok`; (b) `assert floyd_warshall(mut d)`, which may be skipped in release; (c) a `lint(discarded_result: allow)` block.
- Chose: (a).
- Cost if wrong: (b) would skip the whole computation in release if asserts are compiled out. (a) at worst leaves an unused-value warning.

### H-TAG-RANGE-READ
- Question: does an enum read with `@tag(1) Add(r: Range<V>, u: ...)` read the tag token and then `a b` as the inclusive `a..=b`, the way `Range<T>` input does?
- Readings: (a) yes, `Range<T>` reads two inclusive ends in any position; (b) a `Range` field inside a variant needs explicit `l: V, r: V`.
- Chose: (a).
- Cost if wrong: an off-by-one at the right end of every update, or a compile error.

### H-TREE-EDGES
- Question: what is the idiomatic way to read a tree's n-1 edges?
- Readings: (a) `domain E = 0..V.count - 1` then `read<[E](u: V, v: V)>()`; (b) `read<[V.count - 1](u: V, v: V)>()`; (c) a validated `Tree<V>` read, which is mentioned but has no read rule I could find.
- Chose: (a) (06-tree-matching.cp).
- Cost if wrong: only style if (b) is the norm. If (c) is the norm, the input is not validated as a tree.

### H-GRID-FIND
- Question: how do you find the cell holding 'A' in a `[R, C]Char` grid?
- Readings: (a) `grid.keys.find(p => grid[p] == 'A')` (07-sequences line 287 shows the 1-D form); (b) `(R, C).find(...)` on the product domain (R04.2.20 says a domain is a `Seq`, row-major for products, but does not show an inline `(R, C)` as an expression).
- Chose: (a).
- Cost if wrong: a compile error if `.keys` on a 2-D array is not a `Seq<(R, C)>`.

### H-BFS-LAMBDA-VERTEX
- Question: with `bfs(p => p.adj4.filter(...), from: [a], until: b)`, is the vertex type the product position `(R, C)` inferred from `from:`, and do dense `[V]` tables over a product domain work?
- Readings: (a) yes (graphs-trees line 143 shows an implicit grid); (b) the vertex must be a single domain, and you build a `Graph` over a flattened cell domain.
- Chose: (a) (01-labyrinth.cp).
- Cost if wrong: you rewrite the grid as an explicit graph.

### H-TUPLE-FIELD
- Question: are the parts of a product position `(R, C)` reached with `.0` and `.1`?
- Readings: (a) `.0` / `.1`; (b) only destructuring, `let (r, c) = p`; (c) named fields.
- Chose: (a) in `move`.
- Cost if wrong: a compile error. The fix is local.

### H-FOR-IF
- Question: is `for c in coins if c <= s:` a legal filtered loop header?
- Readings: (a) yes, like the `if` clause of comprehensions; (b) only comprehensions take `if`, so a loop needs `coins.filter(...)` or an inner `if`.
- Chose: (a).
- Cost if wrong: a compile error. The fix is one line.

### H-GEN-SORTED
- Question: can a generator such as `chain(a, b)` (std/seq.cp:1623, `-> yields T`) be the receiver of `.sorted()` (std/seq.cp:273, `self: S` with `S: Seq`)?
- Readings: (a) generators are `Seq` and materialize when sorted; (b) you need `List(chain(...))` first.
- Chose: (a) (03-restaurant-customers.cp).
- Cost if wrong: a compile error.

### H-KEEP-RECORD
- Question: does the record returned by `keep: r => (to: r.to, w: Int(r.w))` satisfy both `Target<V>` and `Weighted<W>` structurally, so that `matrix()` accepts it?
- Readings: (a) yes, field traits are structural; (b) you must use the stdlib arc type or `.weighted(...)`.
- Chose: (a).
- Cost if wrong: a compile error at `matrix()`.

### H-RECORD-PATTERN
- Question: is `for (u:, v:) in read<[q](u: V, v: V)>()` the right record destructuring in a loop header?
- Readings: (a) `(u:, v:)` shorthand; (b) positional `(u, v)`; (c) bind `e` and use `e.u`.
- Chose: (a).
- Cost if wrong: a compile error.

### H-DECL-ORDER
- Question: can a top-level `enum` mention `V` before the `domain V = ...read...` line, given that statements run in order but type declarations might be hoisted?
- Readings: (a) type declarations see every top-level domain; (b) the enum must come after the domain.
- Chose: (a), with the enum first (05-range-update-queries.cp).
- Cost if wrong: a compile error. The fix is to move the declaration.

### H-ROOTED-TEMP
- Question: may `Rooted(...)` take a temporary `Graph` and keep using it after the expression ends?
- Readings: (a) `Rooted` builds its own tables, so the temporary is fine; (b) it borrows the graph, so the graph must be a named `let`.
- Chose: (a).
- Cost if wrong: a borrow error. The fix is one line.

### H-Z-ZERO
- Question: what is `z_function(s)[0]`: 0 or `s.len`? And is `z[s.len]` a trap?
- Readings: (a) `z[0] = 0`, as the stub leaves it; (b) `z[0] = n`.
- Chose: never read `z[0]`, and guard `p == s.len` with `or` before `z[p]`.
- Cost if wrong: none for this program. Code that reads `z[0]` gets a different answer under each reading.

### H-PRINT-POSITIONS
- Question: does `print` of a `List<V>` or `[V]C` print each position's integer value?
- Readings: (a) yes, `Int(v)`; (b) its 0-based ordinal; (c) positions are not `Show`.
- Chose: (a) for `topo` and (a) plus 1 for `comp`.
- Cost if wrong: under (b), Course Schedule prints labels off by one.

### H-MOD-CTOR
- Question: `Mod<P>` has a field constructor `Mod(v: 0..P)` and `pub fn new(x: Int)`. Does `Z(a)` call the range-checked field form or `new`, which reduces modulo P?
- Readings: (a) `new`, by the custom-constructor rule; (b) the field form, which traps outside `0..P`.
- Chose: `Z(a)` with `a <= 1e9 < P`, which is safe under both.
- Cost if wrong: none here. A trap under (b) whenever `a >= P`.

### H-TUPLE-READ
- Question: does `read<(0..=1e9, 0..=1e9, 0..=1e9)>()` read three tokens into a tuple that `let (a, b, c)` can destructure?
- Readings: (a) yes; (b) you must read three times, or read a record.
- Chose: (a).
- Cost if wrong: a compile error. The fix is one line.

### H-STAMP
- Question: should a file open with the `// cpl 0.3.0` version stamp, given that the task asks for a two-line problem/I-O header?
- Readings: (a) the stamp is optional; (b) the stamp is required as the first line.
- Chose: (a), no stamp.
- Cost if wrong: a version warning or error on every file.

### H-FN-PLACEMENT
- Question: in a flat top-level program, can a `fn` sit between statements and capture an earlier top-level `let`?
- Readings: (a) yes, like a nested function; (b) functions must come first and take only parameters.
- Chose: (a) (`move` in 01-labyrinth.cp, which uses only its parameters).
- Cost if wrong: small. The fix is to move it.

## Contradiction

### H-RANGEFENWICK-GONE
- Question: is there a Fenwick for range add?
- Readings: spec/lib/ops-ranges.md lines 405-416 ("What merges") say `RangeFenwick` goes, `DualSegTree` becomes `SegTree` with `act:`, and range add with point read is `SegTree(a: a, act:)` (table line 398). std/fenwick.cp still defines `DualFenwick` (line 257, range add, point get) and `RangeFenwick` (line 326), and std/README.md line 999 lists `DualFenwick` as `[new]`.
- Chose: the spec's `SegTree(a: ..., act: add)`.
- Cost if wrong: only style if `DualFenwick` is the blessed form. A contestant who follows the stdlib may build on a type that the spec says is gone.

### H-SCC-BLOCKED
- Question: do the free graph functions `scc`, `topo` and `components` accept a graph over a user domain `V`?
- Readings: spec/lib/graphs-trees.md line 224 documents `scc(g)` as returning `(domain C, comp: [V]C)`. std/graph.cp lines 465, 548 and 640 carry the comment "blocked: generic `V` of a `Next<V>` free fn is bound to `Int`".
- Chose: the spec (08-course-schedule.cp and 10-planets-and-kingdoms.cp use them over `domain V`).
- Cost if wrong: both programs fail to compile, and a contestant has no documented fallback.

### H-STD-K-COUNT
- Question: what is the size of a domain, and how do you get a position's offset?
- Readings: spec/core/04-indices.md line 28 (R04.2.2) uses `V.count` and `Int(v)`. std/fenwick.cp lines 23 and 29 and std/graph.cp (`let n = V.K` in `directed`) use `D.K` and `i.to_idx`, which the spec does not define.
- Chose: `V.count` and `Int(v)`.
- Cost if wrong: a contestant who copies stdlib code gets names the spec does not define.

## Missing stdlib

### H-DIR-FROM-STEP
- Question: to print a grid path as moves, you need the direction between two adjacent cells. std/grid.cp has `Dir4` and `step`, and a letter-to-direction mapping, but no direction from a pair of cells and no direction to letter.
- Readings: (a) there is no such name, so write a `match` on the coordinate difference; (b) a `Dir4.between(a, b)` / `.letter` that I could not find in std/.
- Chose: (a), a hand-written `move` (01-labyrinth.cp).
- Cost if wrong: about six lines, and a chance to swap U/D or L/R.
