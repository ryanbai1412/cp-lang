# Rebuild round 31: holes (cold)

Written from `spec/` and root `std/` only, before running any tool. The
"Held?" column and the found-late section are filled in after verification.

## Problems

| # | CF | Rating | Slot | Main facilities |
|---|---|---|---|---|
| 01 | 1898E Sofia and Strings | 2200 | string (substitute, see below) | `Str`, per-letter lists, `@!multitest` |
| 02 | 1858D Trees and Segments | 2200 | two pointers via `maximal` over `Window(+)` | `maximal`, `Anchor.Start`, `[J, P.cuts]` |
| 03 | 1866D Digital Wallet | 2300 | DP over (column, ops used) | `[R, Col]` read, `sorted(by: desc)`, `scan(+)`, `max=` on `?Int` |
| 04 | 1881G Anya and the Mysterious String | 2000 | Fenwick with range updates | `DualFenwick.add(range)`, `Fenwick.set/fold` |
| 05 | 1881F Minimum Maximum Distance | 1700 | tree with `Rooted` (rerooting) | `Rooted`, `reroot(max, lift:, finish:)` on `?Int` |
| 06 | 1879D Sum of XOR Functions | 1700 | DP over bitmasks (per-bit parity state) | `scan(^)`, cuts, `Mod<P>` |
| 07 | 1866H Happy Sets | 2100 | combinatorics with `Mod<P>` and `Comb` | `Comb.arrange`, `Comb.fact`, `**` |
| 08 | 1878F Vasilije Loves Number Theory | 1900 | multitest, shared precomputation | `Sieve` above `@!multitest`, `Counter`, `.clone()` |
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

## Counts (cold)

| Class | Count | Inside M1 |
|---|---|---|
| silent | 22 | 21 |
| contradiction | 3 | 3 |
| missing-stdlib | 2 | 2 |
| found-late | (after run) | |

M1: every family used here is `supported` in `spec/SUBSET.md` (seq, strings,
fenwick, tree, window, mod, math, hashmap, graph, io); `std-subset` is
`documented` but no solution uses `Subset`. S21 (inline `fn` capturing a
top-level `var`) is core 06, also supported; S12 is the one I place outside
because explicit type arguments are a listed gap in `std/GAPS.md`.

## Silent holes

Each: what the spec did not say, the guess written, and whether it held.

| id | Problem | Spec silent on | Guess | M1 | Held? |
|---|---|---|---|---|---|
| S1 | 01, 02, 04 | whether `Str` is indexed by `Int` and iterable by `Char` | `s[i]`, `for ch in t` | yes | |
| S2 | 01 | filling a domain array with a heap value per key | `[L] _ => List<Int>()` | yes | |
| S3 | 01 | a filter on a `for` over a whole domain, and `<` on positions | `for d in L if d < c` | yes | |
| S4 | 01, 04 | `Char` to `Int` conversion | `Int(c) - Int('a')` | yes | |
| S5 | 02 | a product array whose second axis is a cuts domain | `[J, P.cuts]Int` | yes | |
| S6 | 02 | iterating a domain backwards | `for i in P.rev` | yes | |
| S7 | 02 | the last cut of a domain | `P.cuts.last!` | yes | |
| S8 | 02 | the `ok` lambda type for `maximal(Window(+))` over `[P]Int` | `w => (w.all ?? 0) <= j` | yes | |
| S9 | 03 | reading a 2D grid of integers row by row | `read<[R, Col]1..=1e6>()` | yes | |
| S10 | 03 | `scan(+)` on a `List` index type (count of first elements) | `top[t]` sum of first `t` | yes | |
| S11 | 03 | reversing an `Int` range | `(a..=b).rev` | yes | |
| S12 | 04 | an empty `Fenwick` of a given domain and value type | `Fenwick<P, Int>()` | no | |
| S13 | 04 | half-open `l..r` between two positions, and `P(Int(r) - 1)` | position ranges | yes | |
| S14 | 05 | `reroot` with `max` as the op on `?Int` | `None` is the identity (core 02 says so for `max`; reroot says "a Monoid, or an operator") | yes | |
| S15 | 05, 03 | `.map` on `?T` | `x.map(d => d + 1)` | yes | |
| S16 | 06 | `^` as an op value for `scan` | `a.scan(^)` | yes | |
| S17 | 06 | `Int(c)` of a cut, and `Prefix[c]` by cut | cut value = its index | yes | |
| S18 | 08 | assigning through `Counter[k]` (spec shows only `seen[k]` reads and `push`) | `cur[p] += e` | yes | |
| S19 | 09 | `print` of a `List<Char>` | space-separated letters | yes | |
| S20 | 10 | the order fields of a record literal are evaluated in, with `read()` in two fields | left to right | yes | |
| S21 | 04 | a top-level `fn` reading a top-level `var` (`shift`) declared below `@!multitest` | allowed, sees current value | yes | |
| S22 | 10 | `DualFenwick(tree.reorder(...))` taking `t.sub(v)` (a `Range<Pre>`) | the domain is inferred as `Pre` | yes | |

## Contradictions

| id | Where | What | M1 |
|---|---|---|---|
| C1 | `spec/lib/sequence-patterns.md` vs `std/window.cp` | the spec says `Window(op)` picks its strategy from laws (monotone deque for selective ops); `std/window.cp` is always two stacks. Same amortized cost, but "name the algorithm" is not what the code does. | yes |
| C2 | `spec/lib/graphs-trees.md` vs `std/tree.cp` | `t.up_arc(v)` and `t.child_arcs(v)` are documented; their bodies are `todo`. | yes |
| C3 | `spec/lib/graphs-trees.md` vs `std/GAPS.md` | the spec presents `Rooted` and `Graph` generic over a domain `V`; GAPS says `cpl run` resolves them to built-ins typed over `Int` (E0302 on `?V`). 05 and 10 may fail. | yes |

## Missing stdlib

| id | Problem | Missing | Workaround | M1 |
|---|---|---|---|---|
| M1 | 08 | `Counter` has no `add(k, x)` for a count other than 1 (`push` adds 1) | S18 guess | yes |
| M2 | 08 | `Int` power under a runtime modulus as a function (`Mod(p)` needs a type per value of `d`) | multiply `e` times | yes |

## Found late

(after verification)

## Top five risks (cold)

1. C3: `Rooted`/`Graph` over a declared domain may not run at all in `cpl run` (05, 10).
2. S5/S17: cuts as an array axis and as `Int`; the spec talks about cuts a lot but never in a product.
3. S12/S22: generic constructors inferring their domain (`Fenwick<P, Int>()`, `DualFenwick(tree.reorder(...))`).
4. S1: `Str` indexing and iteration; every string solution leans on it.
5. S18/M1: `Counter` writes through `[]`.

## Revision test

| Problem | Change | Lines changed | Chunks | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|---|
| 06-1879D (DP) | weight `(r - l + 1)` becomes `(r - l + 1)^2` | 4 changed, 3 added (plus comment) | 1 (inner loop) | where the sum of squares fits the Z conversions | none | none |
| 10-1891F | new op `3 v`: print `v`'s value at that moment | 2 changed, 2 added (plus comment) | 2 (read, replay) | whether `elif` exists (it does, core 01) | the op tag `3` is written in the read and in the replay | none |
