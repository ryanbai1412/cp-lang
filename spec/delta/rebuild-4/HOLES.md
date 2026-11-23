# Rebuild round 4: holes

I read `ryanbai1412/cp-lang` at commit `f6ff5434d19e19fb709400ea0f22df2c0f25e9bb` (`f6ff5434d`), and only `spec/`: `spec/README.md`, `spec/core/`, `spec/lib/`, `spec/tools/`, `std/` and `spec/diagnostics.md`. I did not open `spec/delta/`, `spec/conformance/` or anything outside `spec/`, and I did not run the compiler, so every program here is checked against the text alone.

## Summary

There are 20 holes: 13 silent, 5 contradiction, 2 missing-stdlib.

The ten most likely to hit a contestant, most likely first:

1. H-PATHS-DIST-SPELLING (contradiction): `p[v]` or `p.dist[v]` for a shortest-path distance.
2. H-ANON-ARRAY-INT-INDEX (contradiction): can a `[n]T` array be indexed by a plain `Int`?
3. H-AND-SHORT-CIRCUIT (silent): does `and` skip its right side?
4. H-REVERSED-RANGE (silent): is `a..=b` with `a > b` empty or a trap?
5. H-BORDERS-NOT-IN-PRELUDE (contradiction): does `borders` need a `use`?
6. H-RANGE-WEIGHT-SUM (silent): does a `w: 1..=1e9` field make distances a `1..=1e9` type?
7. H-DFS-STACK-DEPTH (silent): is `dfs` (and building a `Rooted`) recursive?
8. H-OPT-ARRAY-CHAIN (missing-stdlib): `from.chain(v)` has no stub and no stated order.
9. H-COUNTED-DIM-IDENTITY (silent): are two `[n]T` arrays over the same index type?
10. H-DRAIN-WALK (silent): how to run a walk only for its effect on `seen`.

## Silent

### H-AND-SHORT-CIRCUIT

- Question: does `a and b` evaluate `b` only when `a` is true? The two-pointer loop `while j < m and size[j] < w - k` relies on it, since `size[j]` traps when `j == m`.
- Readings: (a) short-circuit, as in Python, C++ and Rust; (b) both sides evaluated. `core/01-lexical-grammar.md` lists `and`/`or`/`not` as operators but no file in `spec/` says when the right side is evaluated.
- Chose: (a).
- Cost if wrong: an index trap (R0101) on the last step of every two-pointer and guarded-lookup loop. The fix is a nested `if`.

### H-REVERSED-RANGE

- Question: what is `lo..=hi` when `lo > hi`, as in `(price[b]..=x).rev` when a book costs more than the budget?
- Readings: (a) an empty range that loops zero times; (b) a trap; (c) a compile or lint diagnostic. `core/04-indices.md` (Ranges, R04.4.12) defines `r.len` and `r.is_empty` but never says whether a backwards pair of ends builds an empty range. `core/10-io.md` R10.2.3 only says that *reading* a `Range<T>` checks `l <= r`.
- Chose: (a).
- Cost if wrong: knapsack-style loops trap on input where an item is larger than the capacity. The fix is an `if price[b] <= x` guard.

### H-RANGE-WEIGHT-SUM

- Question: in `Graph.directed(read<[m](u: V, v: V, w: 1..=1e9)>())` followed by `dijkstra`, what is the distance type `W`? Sums of weights exceed `1e9` right away.
- Readings: (a) `W` is the field type `1..=1e9`, so `dist: [V]?W` traps on the first long path (R0201); (b) arithmetic is `Int` by default (`core/02-values-types.md` R02.2, `core/03-numbers-layout.md`), so `W` is `Int` and the range is only a read contract; (c) the type is widened only where the sum is stored. `std/graph.cp:1216` `dijkstra<V, W, G: WeightedNext<V, W>>` takes `W` from the arcs and says nothing about how a range-typed weight is widened. `core/00-overview.md:44-46` uses exactly this pattern.
- Chose: (b), and copied the spec's own pattern.
- Cost if wrong: every weighted shortest-path program traps, or stores distances in a width that is too narrow. The fix is to read `w: Int`, which throws away the contract.

### H-DFS-STACK-DEPTH

- Question: is `dfs` (`std/graph.cp:1890`) recursive, and is `Rooted.from_parents` (`std/tree.cp:100`)? A 1000 × 1000 snake-shaped room or a 2·10^5-vertex path tree goes 10^6 or 2·10^5 deep.
- Readings: (a) iterative, so any depth is safe; (b) recursive, so R0601 stack exhaustion is possible on the judge. The docs for other walks in `std/graph.cp` say "without recursion" (lines 548, 688, 717, 1402, 1524). The `dfs` doc only says "O(n + m) total, O(1) per step". `tools/judge-profiles.md` fixes a stack size, but nothing ties that size to `dfs`.
- Chose: (a).
- Cost if wrong: a runtime error on the largest tests only, which is the worst kind to debug in a contest. The fix is a hand-written explicit stack or `bfs`, which means rewriting the flood fill.

### H-DRAIN-WALK

- Question: how do you run a generator only for its side effect? Here `dfs(..., seen: mut seen)` is run only to mark one room.
- Readings: (a) `for _ in walk: ()`; (b) `walk.count()`; (c) a dedicated `.run()` or `.drain()`. `core/07-sequences.md` describes lazy views and materialization but no "consume and discard". The `dfs` doc (`std/graph.cp:1880-1889`) says `seen` "is read and updated in place" but not when that happens if nothing iterates the walk.
- Chose: (a). It is the plainest form that is certain to walk the whole generator, if `()` is a valid statement body.
- Cost if wrong: a syntax error, or, if the walk runs lazily and is never consumed, every cell counts as its own room (a silent wrong answer).

### H-NEXT-VERTEX-INFERENCE

- Question: when `dfs` gets a lambda `p => p.adj4.filter(...)` as `G: Next<V>`, where does `V` come from (here, the product domain `Cell`), so that `seen: [V]Bool` is dense?
- Readings: (a) from `from: [p]`; (b) from the lambda's parameter, which then needs an annotation; (c) from `seen:`. `lib/graphs-trees.md:138` passes lambdas to `dijkstra` but always with a typed `s` in `from:`. No rule in `core/06-control-functions.md` states how inference works across arguments.
- Chose: (a), with no annotation.
- Cost if wrong: a type error, or `V` inferred as something unbounded, which then needs `hashed: true`. The fix is a one-line annotation.

### H-PRODUCT-POSITION-IS-TUPLE

- Question: is a position of `domain Cell = (Row, Col)` the same type as the tuple `(Row, Col)`? `adj4` is declared on `self: (R, C)` (`std/grid.cp:181`), and the grid is read as `[Row, Col]Char` but indexed `g[q]` with `q: Cell`.
- Readings: (a) yes, fully interchangeable; (b) a `Cell` position destructures as a tuple but is a distinct type, so `adj4` yields `(Row, Col)`, which then needs `Cell(...)` to use as a `Cell`. `core/04-indices.md:18` says `[Cell]T` and `[Row, Col]T` "are one 2-D array type", and R04.2.10 (lines 37-38) says a position "is a tuple of factor positions". Neither says whether the *position types* are equal.
- Chose: (a).
- Cost if wrong: type errors in every grid BFS or DFS that mixes `adj4` with a `Cell`-keyed array.

### H-COUNTED-DIM-IDENTITY

- Question: are `read<[n]Int>()` and a second `read<[n]Int>()` arrays over the *same* index type `0..n`, so that a position of one indexes the other?
- Readings: (a) yes, `[n]T` is always `[0..n]T` (R02.8.1, `core/02-values-types.md:191`), so the two match structurally; (b) every anonymous range is its own type, like a fresh `domain`. Neither file says whether two anonymous ranges with equal ends are one type.
- Chose: neither. I declared `domain Book = 0..n`, which the text makes safe.
- Cost if wrong: under (b), parallel input arrays such as `prices` and `pages` cannot be zipped by position (E0301). Under (a), the protection that named domains give is lost without any warning.

### H-DOMAIN-DROP

- Question: how do you loop over all of a domain except its first position (employees 2..n)?
- Readings: (a) `Emp.drop(1)`, if a domain is `Indexed` (`std/seq.cp:1388` declares `drop` on `S: Indexed`); (b) `for e in Emp.first!.next!..` (`core/04-indices.md:142-143` says "write `for j in v.next..` only when `v.next` exists"); (c) a guard inside the loop. R04.2.20 (`core/04-indices.md:29`) makes a domain a `Seq`, but not an `Indexed`.
- Chose: (c), `if e != Emp.first!`.
- Cost if wrong: low. The cost is a clumsier loop, not a wrong answer.

### H-ENUM-RUNTIME-DOMAIN-FIELD

- Question: can a top-level `enum` declare payload fields of a domain whose size is read at run time (`Update(k: Pos, u: Int)` with `domain Pos = 1..=n`)?
- Readings: (a) yes, as in the `@tag(1) Add(l: V, r: V)` example (`core/02-values-types.md:172-173`, R02.7.2); (b) no, because a type declaration cannot depend on a run-time value, or because of E0313 `domain_escapes` (`diagnostics.md`). The example never says whether its `V` was sized at run time.
- Chose: (a).
- Cost if wrong: the command-stream idiom (`read<Cmd>()` in a loop, R10.2.3) stops working for 1-based query fields. The fix is to read `Int`s and convert with `Pos(x)`.

### H-VARIANT-NAME-COLLISION

- Question: can an enum variant use the name of a builtin or prelude type, for example `Set(k, u)` for an "assign" query?
- Readings: (a) yes, since variants live in their enum's scope (R02.7.1, `core/02-values-types.md:167`); (b) a collision error; (c) the bare name resolves to the type in expression position. R02.7.1 covers patterns and expected types, but not a clash with a prelude type name such as `Set`.
- Chose: avoided the clash by naming the variants `Update` and `Ask`.
- Cost if wrong: a confusing name-resolution error on the most natural variant names (`Set`, `Add`, `Min`).

### H-EMPTY-SEQ-PRINT

- Question: what does `print(xs)` print when `xs` is empty, as when a string has no borders: an empty line or nothing?
- Readings: (a) just the newline; (b) nothing. R10.3.1 (`core/10-io.md:82`) says `print(xs)` "prints a sequence on one line" and `print` "ends the line", but does not cover the empty case.
- Chose: (a).
- Cost if wrong: usually harmless on whitespace-tolerant judges, but a presentation error on strict checkers.

### H-STAMP-LINE

- Question: R11.4.1 (`core/11-modules-prelude.md:80`) says "The first line of a solution is a compiler stamp, added by the formatter: `// cpl 0.3.0`". Is a hand-written file without it valid, and where does `cpl fmt` put the stamp when the file already starts with a header comment?
- Readings: (a) the stamp is optional and the formatter prepends it; (b) the stamp is required and its absence is a diagnostic; (c) the formatter replaces the first comment line.
- Chose: (a). The files start with the requested two-line header and have no stamp.
- Cost if wrong: under (b), an error on every hand-written file. Under (c), the problem name is silently lost.

## Contradiction

### H-PATHS-DIST-SPELLING

- Question: how do you read the distance to `v` from the `Paths` that `dijkstra` returns?
- Places that disagree: `core/00-overview.md:47` writes `print(p.dist[V.last!] ?? -1)`, a field access on `dist`. `lib/graphs-trees.md:164` (shortest-path table) and the `get` doc at `std/graph.cp:1038-1042` say `p[v]` is "the distance to `v`" (`?W`). `spec/` never says how `[]` reaches `get`, and `std/GAPS.md:42,51` notes that structs cannot implement indexing.
- Readings: (a) `p[v]`; (b) `p.dist[v]`; (c) `p.get(v)`.
- Chose: (a), since the library chapter and the stub agree.
- Cost if wrong: the first Dijkstra program fails to compile, either on `p[v]` (not indexable) or on `p.dist` (private field), and the contestant cannot tell from the spec which one is right.

### H-ANON-ARRAY-INT-INDEX

- Question: can an array over an anonymous range (`[n]Int`, `[0..=x]Int`) be indexed by a plain `Int`?
- Places that disagree: `core/02-values-types.md:188-191` (R02.8.1) says "`[D]T` is a fixed-size array indexed by domain `D` (positions of `D`, never raw `Int`)" and "`[n]T` means `[0..n]T`". `core/04-indices.md:77-78` (R04.3.3) says "Arrays over an anonymous range (`[0..=n]Int`) take any `Int`, checked".
- Readings: (a) R04.3.3 governs anonymous ranges and R02.8.1 governs named domains; (b) R02.8.1 is absolute, so every `Int` needs converting.
- Chose: (a), for `best[c]` in knapsack and `size[j]` in two pointers.
- Cost if wrong: every DP table and two-pointer loop fails with E0301 until `D(x)` conversions are added, and an anonymous range has no name to write `D(x)` with.

### H-BORDERS-NOT-IN-PRELUDE

- Question: is `borders` in the prelude?
- Places that disagree: `core/11-modules-prelude.md:53-60` (R11.3.2) says "every pub type and free function in a chapter's main tables is in it, and nothing else is". `lib/strings.md:167` lists `fn borders<S: Indexed>(s: S) -> yields Int` in the same §3.2 table as `prefix_function` and `z_function`. `std/prelude.cp` imports `prefix_function` (line 106) and `z_function` (line 110) but not `borders`.
- Readings: (a) `borders` is in the prelude per R11.3.2, and `prelude.cp` is wrong; (b) `prelude.cp` "is the authoritative list" (same rule), so a `use` is needed.
- Chose: (b), `use std.strings.{borders}`.
- Cost if wrong: under (b) with no `use`, an unknown-name error. Under (a) with the `use`, possibly a redundant-import lint. Both are cheap but cost a compile cycle.

### H-PREFIX-FUNCTION-RETURN

- Question: what does `prefix_function` return (considered for Finding Borders before choosing `borders`)?
- Places that disagree: `lib/strings.md:164` says `-> List<Int>`. `std/strings.cp:415` says `-> [K]Int` (`failure_lengths(s).rekey(s.keys)`), keyed by the string's own keys.
- Readings: (a) `List<Int>`, indexed `0..len`; (b) `[K]Int`, indexed by `s`'s keys. The two differ for a slice `s[l..]`, where the keys keep their source offsets (R04b).
- Chose: did not depend on it, because I used `borders`. A prefix-function solution would follow the stub.
- Cost if wrong: an off-by-offset bug, or E0301, when running KMP on a slice.

### H-IO-BASE-STATUS

- Question: is `with io(base: 1)` a usable way to take 1-based input?
- Places that disagree: `core/10-io.md:47-50` (R10.2.2) and `core/09-contexts-checks.md:25` present `io(base: 1)` as working. `core/12-powers.md:315` says "`with io(base: 1)` and friends wait for the program-shape design".
- Readings: (a) available now; (b) not yet specified, so write `domain V = 1..=n`.
- Chose: (b), the form that `lib/graphs-trees.md` and `core/04-indices.md:15` use.
- Cost if wrong: low here. The cost is a choice between two styles that the spec does not settle.

## Missing stdlib

### H-OPT-ARRAY-CHAIN

- Question: what is `from.chain(v)` on a back-pointer table `from: [D]?D`? `lib/search-dp.md:83` names it as the reconstruction idiom ("walks the kept argmins: `for c in from.chain(Pos.cuts.last!)`").
- Stubs: no `chain` method on `[D]?D` exists anywhere in `std`. The only `chain`s are the free `chain(a, b)` sequence concatenation (`std/seq.cp:1584`, prelude) and `Rooted.chain` (heavy chain, `std/tree.cp:841`). The yield order and whether the last vertex (the one with `None`) is included are unstated.
- Readings: (a) it yields `v, from[v], from[from[v]], ...` through the vertex whose entry is `None`; (b) the same without `v`; (c) the same in root-first order; (d) it does not exist, so write the loop by hand.
- Chose: (a), then `.collect().rev` to print the route from city 1 to city n.
- Cost if wrong: the reconstruction is reversed, or off by one vertex (a wrong answer), or it fails to compile. Every "DP with path" problem hits this.

### H-SET-MAP-STUB

- Question: what is the API of the builtin `Set`/`Map`/`HashSet`/`HashMap` (constructor from a sequence, `.len`, `insert`)? This matters for the "set" form of Distinct Numbers.
- Stubs: `lib/containers.md:49,158-163` calls them builtin ("Write `Map` and `Set` for a lookup table"). `std/seq.cp:350` (`HashSet<K>()`) and `std/hashmap.cp:483` (`Set<Int>()`) use them. No `struct Set`/`HashSet`/`Map`/`HashMap` stub exists in `std`.
- Readings: (a) `Set(a).len`; (b) `var s = Set<Int>(); for x in a: s.insert(x)`; (c) avoid them and use `distinct` (`std/seq.cp:349`, prelude, documented O(n) expected, with salted hashing per `lib/containers.md:163`).
- Chose: (c), `read<[n]Int>().distinct().len`.
- Cost if wrong: any program that writes `Set`/`Map` directly is guessing method names. A wrong guess is a compile error on the most common container in contest code.
