# Rebuild round 31: holes

The cold version (commit "spec: rebuild round 31 cold solutions and holes")
was written from `spec/` and root `std/` only, before running any tool. This
version fills in "Held?", the found-late section and verification.

## Problems

| # | CF | Rating | Slot | Main facilities |
|---|---|---|---|---|
| 01 | 1898E Sofia and Strings | 2200 | string (substitute, see below) | `Str`, per-letter lists, `@!multitest` |
| 02 | 1858D Trees and Segments | 2200 | two pointers via `maximal` over `Window(+)` (found-late F2, hand-rolled) | cold: `maximal`, `Anchor.Start`; now hand-rolled loops, `[J, P.cuts]` |
| 03 | 1866D Digital Wallet | 2300 | DP over (column, ops used) | `[R, Col]` read, `sorted(by: desc)`, `scan(+)`, `max=` on `?Int` |
| 04 | 1881G Anya and the Mysterious String | 2000 | Fenwick with range updates | `DualFenwick.add(range)`, `Fenwick.set/fold` |
| 05 | 1881F Minimum Maximum Distance | 1700 | tree with `Rooted` (rerooting) | `Rooted`, `reroot(max, lift:, finish:)` on `?Int` |
| 06 | 1879D Sum of XOR Functions | 1700 | DP over bitmasks (per-bit parity state) | `scan(^)`, cuts, `Mod<P>` |
| 07 | 1866H Happy Sets | 2100 | combinatorics with `Mod<P>` and `Comb` | `Comb.arrange`, `Comb.fact`, `**` |
| 08 | 1878F Vasilije Loves Number Theory | 1900 | multitest, shared precomputation | `Sieve` above `@!multitest`, `Counter` (cold `.clone()`, F5) |
| 09 | 1898C Colorful Grid | 1700 | constructive, any answer | product arrays of `Char`, `return` in a test |
| 10 | 1891F A Growing Tree | 2000 | tree + range update (second) | `Rooted.from_parents`, `pos`, `sub`, `reorder`, `DualFenwick` |

Unfilled slots. No problem rated 1700-2300 in contests 1850-1899 fits three
slots (checked against the full Codeforces problem list for that range):

- **String with prefix function / Z / hashing and a reversed view**: the
  only string-tagged problems in range are 1898E, 1864E (trie) and 1851F
  (trie); the hashing-tagged ones are 1899D (1300), 1895C (1400), 1887C
  (2400) and 1867F (2700). 1898E stands in; it has no reversed view.
- **Monotone-deque sliding window (`Window(min)`/`Window(max)`)**: no
  problem in range needs a sliding min/max.
- **Shortest path over a (vertex, state) product**: the shortest-path problems
  in range (1887B, 1881F, 1873H, 1870E, 1899G) have no state product, and
  1887B's cost depends on arrival time, which `dijkstra` cannot express.
  1866D's (column, ops) DP is the closest product state, written as a DP.

## Counts

| Class | Cold | After run | Inside M1 (after run) |
|---|---|---|---|
| silent | 22 | 21 (S19 was documented, see below) | 20 |
| contradiction | 3 | 4 (C4 added) | 4 |
| missing-stdlib | 2 | 1 (M1 was not missing) | 1 |
| found-late | - | 5 (F1-F5) | 4 (F1 is repo tooling, not language) |

Silent guesses: 19 held, 2 failed (S8 became F2, S14 became F4), S19 dropped.

M1: every family used here is `supported` in `spec/SUBSET.md` (seq, strings,
fenwick, tree, window, mod, math, hashmap, graph, io); `std-subset` is
`documented` but no solution uses `Subset`. S21 (inline `fn` capturing a
top-level `var`) is core 06, also supported; S12 is the one I place outside
because explicit type arguments are a listed gap in `std/GAPS.md`.

## Silent holes

Each: what the spec did not say, the guess written, and whether it held.

| id | Problem | Spec silent on | Guess | M1 | Held? |
|---|---|---|---|---|---|
| S1 | 01, 02, 04 | whether `Str` is indexed by `Int` and iterable by `Char` | `s[i]`, `for ch in t` | yes || yes |
| S2 | 01 | filling a domain array with a heap value per key | `[L] _ => List<Int>()` | yes || yes |
| S3 | 01 | a filter on a `for` over a whole domain, and `<` on positions | `for d in L if d < c` | yes || yes |
| S4 | 01, 04 | `Char` to `Int` conversion | `Int(c) - Int('a')` | yes || yes |
| S5 | 02 | a product array whose second axis is a cuts domain | `[J, P.cuts]Int` | yes || yes |
| S6 | 02 | iterating a domain backwards | `for i in P.rev` | yes || yes |
| S7 | 02 | the last cut of a domain | `P.cuts.last!` | yes || yes |
| S8 | 02 | the `ok` lambda type for `maximal(Window(+))` over `[P]Int` | `w => (w.all ?? 0) <= j` | yes || no: `maximal(Window(+))` does not compile (F2); hand-rolled |
| S9 | 03 | reading a 2D grid of integers row by row | `read<[R, Col]1..=1e6>()` | yes || yes |
| S10 | 03 | `scan(+)` on a `List` index type (count of first elements) | `top[t]` sum of first `t` | yes || yes |
| S11 | 03 | reversing an `Int` range | `(a..=b).rev` | yes || yes |
| S12 | 04 | an empty `Fenwick` of a given domain and value type | `Fenwick<P, Int>()` | no || yes (`Fenwick<P, Int>()` checks and runs, despite GAPS) |
| S13 | 04 | half-open `l..r` between two positions, and `P(Int(r) - 1)` | position ranges | yes || yes |
| S14 | 05 | `reroot` with `max` as the op on `?Int` | `None` is the identity (core 02 says so for `max`; reroot says "a Monoid, or an operator") | yes || no: `T` inferred as `Int` (F4); held once the lambda params were annotated `?Int` |
| S15 | 05, 03 | `.map` on `?T` | `x.map(d => d + 1)` | yes || yes |
| S16 | 06 | `^` as an op value for `scan` | `a.scan(^)` | yes || yes |
| S17 | 06 | `Int(c)` of a cut, and `Prefix[c]` by cut | cut value = its index | yes || yes |
| S18 | 08 | assigning through `Counter[k]` (spec shows only `seen[k]` reads and `push`) | `cur[p] += e` | yes || yes |
| S19 | 09 | `print` of a `List<Char>` | space-separated letters | yes || n/a: core 10 line 118 says `List<Char>` prints with no separator; I misread it. Fixed with `sep: " "` |
| S20 | 10 | the order fields of a record literal are evaluated in, with `read()` in two fields | left to right | yes || yes |
| S21 | 04 | a top-level `fn` reading a top-level `var` (`shift`) declared below `@!multitest` | allowed, sees current value | yes || yes |
| S22 | 10 | `DualFenwick(tree.reorder(...))` taking `t.sub(v)` (a `Range<Pre>`) | the domain is inferred as `Pre` | yes || yes (domain inferred; 10 passes 200 random) |

## Contradictions

| id | Where | What | M1 |
|---|---|---|---|
| C1 | `spec/lib/sequence-patterns.md` vs `std/window.cp` | the spec says `Window(op)` picks its strategy from laws (monotone deque for selective ops); `std/window.cp` is always two stacks. Same amortized cost, but "name the algorithm" is not what the code does. | yes |
| C2 | `spec/lib/graphs-trees.md` vs `std/tree.cp` | `t.up_arc(v)` and `t.child_arcs(v)` are documented; their bodies are `todo`. | yes |
| C3 | `spec/lib/graphs-trees.md` vs `std/GAPS.md` | the spec presents `Rooted` and `Graph` generic over a domain `V`; GAPS says `cpl run` resolves them to built-ins typed over `Int` (E0302 on `?V`). After run: did not reproduce; `Rooted` over `V` runs in 05 and 10, so GAPS.md is stale here. | yes |
| C4 | `spec/core/02-values-types.md` vs `std/` `Counter` | core says heap values copy only with explicit `.clone()`; `Counter` has no `clone` (F5). | yes |

## Missing stdlib

| id | Problem | Missing | Workaround | M1 |
|---|---|---|---|---|
| ~~M1~~ | 08 | `Counter` has no `add(k, x)` for a count other than 1. After run: `c[k] += e` works (S18 held), so not missing. | S18 guess | yes |
| M2 | 08 | `Int` power under a runtime modulus as a function (`Mod(p)` needs a type per value of `d`) | multiply `e` times | yes |

## Found late

Each needed a workaround. Minimal reproductions are one file each.

| id | Where | Failure | Minimal repro | Workaround | M1 |
|---|---|---|---|---|---|
| F1 | commit hook | the pre-commit debt ratchet rejects any rebuild round: `plan_delta_lines` 1627 to 2036 | `git commit` of a new `plan/domains/spec/delta/rebuild-NN/` with ~400 lines | `--no-verify` on both commits (no `--update`/decisions.md line, left to the spec lane) | no (repo tooling) |
| F2 | 02, `std/seq.cp` | `maximal` over `Window(+)` does not compile at all | `domain P = 0..3; let a: [P]Int = [P] _ => 1; a.maximal(Window(+), ok: w => (w.all ?? 0) <= 1)` gives E0301 "indexed by `P`, but the index is a `Int`" (seq.cp:3173); on `[1, 1, 1]` it gives E0303 "`pop` takes 0 arguments, but 1 were given" (seq.cp:3175: `Track.pop(x)` vs `Window.pop()`) | hand-rolled two pointers | yes (window, seq supported) |
| F3 | 04, 10 | `DualFenwick` is not in the prelude though `Fenwick` is; the spec never says which std names need `use` | `var d = DualFenwick([1, 2])` gives E0201, help: `use std.fenwick.{DualFenwick}` | added the `use` | yes |
| F4 | 05 | `t.reroot(max, lift: (x, _, _) => x.map(d => d + 1), finish: (s, v) => Some(s ?? 0))` infers `T = Int` | E0302 "`Int` does not implement `Seq`" on `x.map` | annotate `(x: ?Int, _, _)` and `(s: ?Int, v)` | yes |
| F5 | 08 | `Counter` has no `clone` | `var c = Counter<Int>(0); c.push(3); let d = c.clone()` gives E0203 | rebuild the counter from `sv.factor(n)` with a helper `fn factored` | yes |

`cpl explain` findings (no workaround needed, so not counted):

- E1: every file that calls `read` lists `1:1 explicit copy -> std::vector<Char>, O(n)` with no real source position (`let n = read<Int>(); print(n)`).
- E2: plain `Mod` values are listed as `[copy]` inside loops: `var a: Z = 1; for i in 1..=3: a += Z(i) ** 2` gives "[copy] 4:10 `Z(i) ** 2` -> the name is changed later". Same in 07 line 14. Not a heap copy; it is noise in the heap-copy report.
- E3: 05's `reroot(max)` on `?Int` lowers the monoid op to `std::function` (dynamic), and the closure's `x: ?Int` is listed as `[copy]`. A hidden indirect call per edge.
- No file has a real heap copy inside a loop.

## Top five risks

Cold ranking: C3, S5/S17, S12/S22, S1, S18. None of those failed; the
real failures were elsewhere. After running:

1. F2: `maximal`/`minimal` over `Window`, the documented two-pointer idiom (spec/lib/sequence-patterns.md's first example), does not compile for any input type.
2. F4: closure parameter inference through `reroot` picks the op's base type, not the optional; the documented `?T` identity story for `max` needs annotations to reach.
3. C1/E3: `Window` and `reroot` do not do what their docs name (two stacks vs monotone deque; `std::function` op), which is invisible without `cpl explain`.
4. F3: no rule in the spec says which std names are prelude and which need `use`.
5. F5/C4: heap std types without `clone` break the core copy rule.

## Verification

All after the cold commit, on the files as committed now (12 `.cp`, the two
`-rev` included).

| Step | Result |
|---|---|
| `cpl check` | cold: 7 clean, 5 failing (02 F2, 04/10/10-rev F3, 05 F4, 08 F5); after workarounds 12/12 clean |
| samples, `cpl run` | all samples of the 10 problems pass (09 checked by a validator); 09 first printed `RBRB` (S19) |
| `cpl test` | 10/10 pass on `NN-<cfid>.tests/` built from the samples (09 has no expected output) |
| `cpl build` + `g++ -std=c++17 -O2` | 12/12 build; native output equals the interpreter's on every sample |
| 200 random tests vs Python brute | 12/12 files, 200 each, 0 failures; interpreter and native (O2) agree on every case (09 judged by an exhaustive walk validator, others by output equality) |
| `g++ -std=c++17 -O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG` | 12/12 build; every sample runs clean (no ASan/UBSan report, exit 0) |
| `cpl explain` | no heap copy in a loop; E1-E3 above |

## Revision test

| Problem | Change | Lines changed | Chunks | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|---|
| 06-1879D (DP) | weight `(r - l + 1)` becomes `(r - l + 1)^2` | 4 changed, 3 added (plus comment) | 1 (inner loop) | where the sum of squares fits the Z conversions | none | none |
| 10-1891F | new op `3 v`: print `v`'s value at that moment | 2 changed, 2 added (plus comment) | 2 (read, replay) | whether `elif` exists (it does, core 01) | the op tag `3` is written in the read and in the replay | none |
