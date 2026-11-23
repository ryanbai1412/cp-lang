# Rebuild round 28: holes (cold read of `spec/` and root `std/` only)

Base: `main` at 68a1a48ef. Statements and samples from the Luogu mirror (Codeforces
returned a bot-check page). Everything above "After running" was written before any
`cargo`/`cpl` command was run.

## Problems

| # | CF | Rating | Coverage | Std used |
|---|----|--------|----------|----------|
| 01 | 1918D Blocking Elements | 1900 | binary search on the answer, window DP | `first_true`, `Prefix`, `Window(min)` |
| 02 | 1921F Sum of Progression | 1900 | 2-D suffix sums over (step, index) | `isqrt`, 2-D arrays |
| 03 | 1930C Lexicographically Largest | 1700 | greedy sorting by a key | `sort(desc:)`, `min=` |
| 04 | 1920C Partitioning the Array | 1600 | number theory (divisors, gcd) | `divisors`, `gcd`, `count` |
| 05 | 1934C Find a Mine | 1700 | interactive + `@interactor` | `Wire`, `@interactor` |
| 06 | 1915F Greetings | 1500 | Fenwick with point updates, compression | `compress`, `Fenwick` |
| 07 | 1915G Bicycles | 1800 | shortest path on a product state (see H09) | `Graph.undirected`, `MinHeap`, `[V, V]?Int` |
| 08 | 1948D Tandem Repeats? | 1700 | string DP (run lengths per shift) | `rev`, `find` |
| 09 | 1925D Good Trip | 1900 | modular overflow care (`k * sum f` exceeds 2^63) | `Mod<P>`, `sum(of:)` |
| 10 | 1945E Binary Search | 1700 | multitest, count line then k swap lines, any answer | `@check`, `read_output`, `swap_at` |

Grid BFS: no unused problem in contests 1900-1949 rated 1500-2100 is a grid BFS or
0-1 BFS (Codeforces tags checked through the API; the grid/graph candidates are all in
earlier rounds). 07 substitutes a Dijkstra over an implicit (city, bike) state, which
exercises the same "search over a product domain" path; this deviation is deliberate.

## Summary counts (before running)

| Class | Count |
|-------|-------|
| silent | 14 |
| contradiction | 2 |
| missing-stdlib | 3 |
| found-late | 0 |
| **total** | **19** |

All 19 are inside the Milestone 1 subset (`spec/SUBSET.md`: core chapters, std-seq,
std-prefix, std-window, std-fenwick, std-heap, std-graph, std-mod, std-math, std-io,
Rstress are all `supported`), except that H09 touches `Graph.undirected`, which std's
own `GAPS.md` lists as not yet expressible as a stub (the subset table still says
std-graph `supported`).

## Top five risks

1. **H09** `Graph.undirected(roads)` over a user domain: `std/GAPS.md` line 12 says
   `cpl run` types `Graph`/`dijkstra` over `Int`; 07 may not check.
2. **H11/H12** interactor harness under `@!multitest`: whether the hidden-test file
   starts with `t`, and what `cpl test` compares for an interactive file.
3. **H06** `rev` is `= todo` in `std/seq.cp:170`; 02, 06, 08 iterate `.rev`.
4. **H07** 2-D arrays with `Int` extents (`[b + 1, n + 1]Int`) and memory of the
   2 x 317 x 1e5 table (about 0.5 GB of `Int`).
5. **H13** `@check fn ok() -> Bool` under `@!multitest`: per-test `read()` of the
   input and `read_output` on that test's output only.

## Holes

| ID | Class | Location | Question | Guess |
|----|-------|----------|----------|-------|
| H01 | silent | spec/core/04-indices.md, spec/core/10-io.md | `read<[n]T>()` with `n: Int`: is the index type `Int` from 0? | Yes, `0..n`, indexable by `Int` arithmetic |
| H02 | silent | std/window.cp:11-52 | `Window(min)` over `Int`: is `pop` FIFO (oldest) and is `min` the free fn accepted as `op`? | Yes; `all` is `?Int` |
| H03 | silent | std/prefix.cp:103-150 | `Prefix(w)` for `w: [k]Int`; `fold(lo..hi)` with `Int` ranges, empty range gives 0 | Yes |
| H04 | silent | spec/core/06-control-functions.md, spec/core/10-io.md | A `fn` below the divider capturing per-test `let`/`var` bindings | Allowed; captures by reference to that test's values |
| H05 | missing-stdlib | std/seq.cp | No "index of first element equal to x" (`position`); wrote `p.keys.find(i => p[i] == x)!` | `find` over `Range<Int>` works |
| H06 | contradiction | std/seq.cp:170 | `rev` is documented in the prelude list but its body is `= todo` | Compiler builtin; works on ranges and lists |
| H07 | silent | spec/core/04-indices.md | 2-D array with `Int` extents `[b + 1, n + 1]Int` indexed `t[d, i]` | Allowed, row-major, heap allocated |
| H08 | silent | spec/core/02-values-types.md | Inline `if c: x else: y` as an expression on one line | Allowed |
| H09 | contradiction | std/graph.cp:201, std/GAPS.md:12 | `Graph.undirected(roads)` over a read record list `[m](u: V, v: V, w: ..)`; arcs give `.to`, `.w` | Works (GAPS describes stub limits, not behaviour) |
| H10 | missing-stdlib | std/graph.cp:1275 | `dijkstra` needs a materialised `WeightedNext`; no form over an implicit state graph (closure `out`) | Hand-rolled Dijkstra with `MinHeap` |
| H11 | silent | spec/lib/random-stress.md:76-86, spec/core/10-io.md:150 | Hidden-test file layout for a multitest interactor: does it start with `t`, read by the harness? | Yes: harness reads `t`, then each interactor call reads one test |
| H12 | silent | spec/tools/cli.md | What `cpl test` compares for a file with an `@interactor` (`N.out` meaning) | Interactor verdict only; `N.out` ignored or transcript |
| H13 | silent | spec/lib/random-stress.md:60-70 | `@check` under `@!multitest`: `read()` sees one test (no `t`), `read_output` that test's output | Yes |
| H14 | silent | spec/core/02-values-types.md | `s[i]` on `Str` returns `Char`; `'?'` literal; `s.len` | Yes |
| H15 | silent | std/mod.cp, std/seq.cp:34 | `sum(of: e => Z(e.f))` over an empty record list gives `Z(0)`; `Z(1) / Z(x)` | Yes |
| H16 | silent | std/seq.cp:1061 | `(0..=1e14).first_true(ok)`: O(log) calls on a huge `Int` range | Yes |
| H17 | silent | std/seq.cp:797-860, std/fenwick.cp:50 | `ends.at(v)` returns `C`; `c.l`/`c.r` are its cuts for `prefix`/`suffix` | Yes |
| H18 | missing-stdlib | std/seq.cp | No "find the first d with Some" returning both key and payload; wrote `first_some(d => f(d).map(st => (d, st)))` | `?T.map` exists |
| H19 | silent | spec/core/01-lexical-grammar.md | `while true:` loop with `return` inside an `@interactor` fn | Allowed |

## Per-problem mapping

| File | Holes |
|------|-------|
| 01-1918D | H01 H02 H03 H04 H16 |
| 02-1921F | H01 H04 H06 H07 |
| 03-1930C | H01 |
| 04-1920C | H01 H04 |
| 05-1934C | H04 H11 H12 H19 |
| 06-1915F | H06 H17 |
| 07-1915G | H08 H09 H10 |
| 08-1948D | H04 H06 H08 H14 |
| 09-1925D | H15 |
| 10-1945E | H01 H05 H13 |
| 06-1915F-rev | H06 H17 |
| 08-1948D-rev | H06 H08 H14 H18 |

## Revision test

| File | Change | Diff | Hesitation | Duplicated fact | Unrelated rewrite |
|------|--------|------|------------|-----------------|-------------------|
| 08-1948D-rev (DP) | Report the leftmost 1-based start of a longest repeat, not only its length | 3 code chunks (fn signature `Bool` -> `?Int`, `return true` -> `return Some(i - d + 1)`, final `print` -> `if let ... first_some`) + header | Choosing between `find` + recompute and `first_some` with a tuple; no `find_map` returning key and value (H18) | None | None: the DP lines are untouched |
| 06-1915F-rev (other) | Answer per person in input order, counting both directions | 4 chunks: `var ps` + `sort` -> `let ps` + `order = keys.sorted(by:)`; the loop goes over indices; a second sweep with `suffix`; `print` an array | Whether to sort records carrying an index or sort indices | `ends.at(ps[i].b)` written twice | Yes: sorting the records in place loses input order, so the first sweep was rewritten over an index list (the `sort` line became `order`) |

## After running

`cargo build -p cpl-cli` (dev), then per file: `cpl check`, `cpl run` and `cpl test` on
the statement samples, `cpl build` + `g++ -std=c++17 -O2` and the native binary on the
samples, 200 random small multitest inputs (1-3 tests each) through `cpl run`, the native
binary and an independent Python brute force, and one maximum-size input on the native
binary. 05 is checked by its own `@interactor` under `cpl run`/`cpl test`, and the native
binary is driven by a separate Python interactor. 10 is compared by a Python checker that
replays the swaps and the search, since any valid answer is accepted.

### Results

| File | check | samples interp | `cpl test` | samples native | 200 random (interp / native vs brute) | max input (native) |
|------|-------|----------------|------------|----------------|----------------------------------------|--------------------|
| 01-1918D | ok | ok | 1/1 | ok | 200/200 | n = 1e5: 0.07 s, 7.6 MB |
| 02-1921F | ok | ok | 1/1 | ok | 200/200 | n = 1e5, q = 2e5: 0.49 s, 507 MB (CF limit 1 GB) |
| 03-1930C | ok | ok | 1/1 | ok | 200/200 | n = 3e5: 0.08 s |
| 04-1920C | ok | ok | 1/1 | ok | 200/200 | n = 166320 (160 divisors), all equal: 0.04 s |
| 05-1934C | ok | interactor accepts | 1/1 (interactor verdict) | Python interactor accepts | 200/200 | t = 3000, n = m = 1e8: 0.31 s wall incl. Python judge |
| 06-1915F | ok | ok | 1/1 | ok | 200/200 | n = 2e5: 0.12 s |
| 06-1915F-rev | ok | n/a | n/a | n/a | 200/200 | not timed |
| 07-1915G | ok | ok | 1/1 | ok | 200/200 | n = m = 1000: 0.02 s |
| 08-1948D | ok | ok | 1/1 | ok | 200/200 | \|s\| = 5000, no repeat: 0.02 s |
| 08-1948D-rev | ok | n/a | n/a | n/a | 200/200 | not timed |
| 09-1925D | ok | ok | 1/1 | ok | 200/200 | n = m = 1e5, k = 2e5: 0.02 s |
| 10-1945E | ok after F01/F02 workarounds | valid (checker) | 1/1 (`@check`) | valid, identical to interp | 200/200 | n = 2e5: 0.01 s |

There were no mismatches between the interpreter and native code, so no sanitizer rerun
was required. As a spot check, 02, 07 and 09 were built with `-fsanitize=address,undefined`
and run on their samples: clean, with identical output.

The 10 checker was tested both ways: `cpl judge` rejects `0` swaps for `6 5 / 3 1 6 5 2 4`
(R0001, exit 3) and accepts a valid output (exit 0). A `@check` that returns `false` also
exits 3, so the earlier note that returning `false` is accepted no longer applies.

### Failures and smallest reproductions

- **F01 (found-late, tool).** `read_output<(1..=n, 1..=n)>()` with an explicit type
  argument fails `cpl check` with E0501 "`read_output` is not in scope during lowering".
  `spec/lib/random-stress.md:64` documents the typed form `read_output<T>()`. The
  workaround `let (i, j): (1..=n, 1..=n) = read_output()` works. Repro: any `@check` body
  containing `let p = read_output<(Int, Int)>()`.
- **F02 (found-late, tool).** Under `@!multitest`, a `@check` whose input reads have a
  different shape from the solution's gets the wrong slice of that test's input. Repro
  (`cpl judge y.cp in out`, in = `1\n6 3\n`, out = `1\n`):
  ```
  @check fn ok() -> Bool:
      let (n, x) = read<(Int, Int)>()
      assert n == 6, "n={n} x={x}"
      true

  @!multitest
  let n: Int = read()
  let x: Int = read()
  print(1)
  ```
  The result is R0501 "the input ended where a number was expected (input line 3)". The
  run passes when both sides use the tuple read, or both use two scalar reads. In 10 the
  same bug read `n = 1, x = 2` from the permutation line, which surfaced as the misleading
  "3 is outside `1..=n`". Workaround: the checker reads `n` and `x` the same way the
  solution does. This contradicts the spec's "runs once per test, on that test's input"
  (`spec/lib/random-stress.md:66`), and it breaks decision 374's intent that a checker
  restates facts independently of the solution.
- **Author errors, not holes.** I wrote `#` comments even though
  `spec/core/01-lexical-grammar.md` lists `#` as rejected (E0004, with a good fix-it). I
  also wrote `read<(1..=2e5, 1..=n)>()`, where the tuple refers to its own binding (a clear
  E0205).

### Guessed holes: outcome

| ID | Held? | Note |
|----|-------|------|
| H01 | held | `[n]T` is `Int`-indexed from 0 |
| H02 | held | `Window(min)` FIFO, `all` is `?Int` |
| H03 | held | `Prefix(w).fold(lo..hi)` over `Int` ranges |
| H04 | held | Fns below the divider see that test's bindings |
| H05 | held (still missing) | `keys.find` works; no `position`/`index_of` |
| H06 | held | `.rev` works on ranges and lists despite `= todo` in `std/seq.cp:170`; the contradiction is in the std source |
| H07 | held | `[b + 1, n + 1]Int` works and is heap allocated (507 MB) |
| H08 | held | Inline `if c: x else: y` |
| H09 | held | `Graph.undirected(roads)` over a user domain checks and runs in both modes, so `std/GAPS.md:12` is stale; the contradiction stands as a doc issue |
| H10 | held (still missing) | Hand-rolled Dijkstra over `[V, V]?Int`; `min=` returns `Bool` on a 2-D optional element |
| H11 | held | Hidden test starts with `t`; the harness reads it and the interactor sees one test |
| H12 | held | `cpl test` on an `@interactor` file needs no `N.out`; the interactor's verdict decides |
| H13 | partly | Per-test slicing works when read shapes match; see F02 |
| H14 | held | `s[i]` is `Char`, `'?'` literal |
| H15 | held | `sum(of:)` over an empty record list is `Z(0)`; `Z(1) / Z(x)` |
| H16 | held | `first_true` over `0..=1e14` |
| H17 | held | `ends.at(v)`, `c.l`, `c.r` with `prefix`/`suffix` |
| H18 | held (still missing) | `first_some` + `?T.map` |
| H19 | held | `while true:` with `return` inside `@interactor` |

### Updated counts

| Class | Before | After |
|-------|--------|-------|
| silent | 14 | 14 (H13 partly wrong, see F02) |
| contradiction | 2 | 2 |
| missing-stdlib | 3 | 3 |
| found-late | 0 | 2 (F01, F02; both tool bugs in `@check`, both inside Milestone 1: Rstress / R10) |
| **total** | **19** | **21** |
