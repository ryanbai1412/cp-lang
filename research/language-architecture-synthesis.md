# CP Lang Architecture Synthesis: Unifying All Scout Friction Backlogs

This document synthesizes every friction data point currently on `main` into one coherent, minimal language and standard-library architecture. It covers all fifteen scout backlogs and the first blind contest-pressure critic audit — Batch 1 (`F1`–`F28`), Batch 2 (`2-B1`–`2-B12`), Batch 3 (`G1`–`G25`), Batch 4 (`4-B1`–`4-B14`), Batch 5 (`K1`–`K21`), Batch 6 (`S1`–`S19`), Batch 7 (`D1`–`D12`), Batch 8 (`H1`–`H21`), Batch 9 (`9-T1`–`9-T18`), Batch 10 (`10-T1`–`10-T12`), Batch 11 (`N1`–`N16`), Batch 12 (`L0`–`L27`), Batch 13 (`R1`–`R11`), Batch 14 (`P1`–`P14`), and Batch 16 (`M1`–`M8`, `Q1`–`Q6`) — totaling 263 numbered items across 151 scout ports and sketches, plus `research/blind-critic-audit-1.md` (8 unseen problems written from the user-facing docs alone, with 4 mid-contest pivots), together with the 30-port baseline (`research/friction.md`, `research/ergonomics-audit.md`) and the 15-port hillclimb pass (`decisions.md` §12). Batch 15 had not landed on `main` at the time of writing. The bug-only prototype sprint (`2b25e6a`, `d6cf87d`, `decisions.md` §15) landed while this was being written; §1 Rank 11 and §7 Phase 1 say which bugs it closed and which are left.

The central finding is striking. **More than four-fifths of the 263 scout items do not ask for new language features.** Almost all of the real friction collapses into three roots:

1. **One structural toolchain split.** `proto/cpc` type-checks and compiles fast C++17 but cannot compile `stdlib/*.cp` or user `struct` methods / `use local.*`, while `proto/cpi` runs `stdlib/*.cp` with no static checker and missing core syntax (`enum`/`match`, `min=` on `?Int`, open slices, `Mod` + `Int` mixing). Nearly every scout batch hits this split first.
2. **Five small, orthogonal extensions to the `plan/19-index-and-lends.md` kernel:** complete algebraic `Index` derivation (finite scalars, `?T`, products `(A, B)` and records, sums via payload `enum`, and exponentials `[D]E`), `let`-frozen parameters generalized from `Index.K` to `Mod<m>`, monotone `Pool<T>` handles with `freeze()` using the same fresh-type binder as `[domain]T`, `F64` (with quiet-NaN niche) plus `U64`/`I128`, and expression/control-flow uniformity (`match`/`if let`/multiline `if` expressions, `for ... if`, `while let ... and`, `??=`, `with` scope guards over `lends ()`, top-level `return`, and `read()` completeness).
3. **Seven cohesive userland `stdlib/` modules:** an order-statistic tree core (`Map`, `Set`, `SortedBag`, `RangeSet`, `Steps`), clearly named binary vs linear search over ranges and indexed collections, a `Window`/`BagWindow` sliding protocol + `Deque`, `std.grid` (`Dir4`/`Dir8`, `step`, `around`, `ray`, `flipped`/`transposed`) built purely on `(Row, Col): Index`, completed `Rooted`/`Hld`/`Graph`/`Dsu`/`LazySegTree`/`Fenwick`/`Subset` algebra + `LiChao`, linear algebra & polynomials (`Semiring`/`Matrix`/`Powers`/`linalg`/`std.poly`/`Sieve.multiplicative`/`Congruence`), and generic `Hash`/`StrHash`/`SuffixArray` + `Pt<T>` geometry.

Everything else in the backlogs is either a concrete prototype bug (23 logged, including an ASan heap-use-after-free in destructuring `let`, since fixed, and a silent wrong answer from `first_where` being a binary search under a linear-search name, still open) or a one-off special case that we should explicitly reject so the core stays small, predictable, and easy to trust under contest pressure.

---

## 1. Recurring Friction Clusters Ranked by Impact

Clusters are ranked by how many problems, scout batches, and CP domains they unlock. Where two batches reused an item prefix, they are disambiguated as `2-B*` vs `4-B*` and `9-T*` vs `10-T*`. Items already fixed in `proto/cpc` or `stdlib/` after an earlier batch are marked so sprint effort goes only to remaining gaps. The "N of 14 batches" reach counts were tallied over batches 1 to 14; batch 16 (`M*`, `Q*`) and the blind critic are folded in by reference in §1.1, §2.3, §3 and §4, and they raise no cluster that was not already ranked.

### Rank 1. Compiler/stdlib split and user structs/local modules (`F1`, `F22`, `H3`, `10-T3`, `9-T1`, `N5`, `N6`, `N14`, `P7`, `P8`, plus interpreter gaps in batches 2, 7, 10, 11)
- **Reach:** 11 of 14 batches, ~55+ scout ports, every CP domain (`struct` alone appears in 255/710 corpus C++ files; 8 of 10 Batch-11 ports could not build with `proto/cpc`).
- **What happens today:** `proto/cpc` cannot compile `stdlib/*.cp`, in-file `struct` methods, or `use local.*` (`F22`, `P7`, `P8`), so every library structure needs a hand-written stand-in in `compiler.py`/`runtime.hpp` that drifts from `stdlib/` (`Hld`, `Comb`, `ModR`, `Bitset`, `LiChao`, `Sieve`, `SegTree.fold`, `Heap`). Meanwhile `proto/cpi` runs `stdlib/*.cp` without static checking or `enum`/`match`/`min=` on `?Int`/open slices/`count`/`[0..n]T`/`Mod + Int` mixing (`N5`, `N6`, `N14`). In Batch 14 alone, missing `struct` methods and `use local.*` cost 155 lines of parallel-array boilerplate across just `ioi16p6io` and `noi19p1`.
- **Why it is #1:** Closing this split once eliminates dozens of duplicate backlog items across every domain and lets every future design change be tested directly on real `.cp` library code.

### Rank 2. Algebraic `Index` completeness — products `(R, C)`, sums (`enum`), finite arrays `[D]E`, ranges (`L5`, `L23`, `4-B1`, `4-B9`, `4-B11`, `G18`, `K11`, `9-T8`, `N1`, `N15`, `R4`, `P5`)
- **Reach:** 9 of 14 batches, every grid, flow, 2-SAT, layered-graph, matrix-state (`noi20p1` `N15`, `noi21p2` `N1`), bitmask, ternary-mask, and board-game DP problem (46 corpus grid-BFS files, 102 `dx`/`dy` grid files, plus all multi-kind graph and k-ary mask DPs).
- **What happens today:** `plan/19` already declared that `Index` is a trait derived for finite types, yet the prototype only lets plain `domain`s sit in `for`, `[I]T`, `Dsu<I>`, `bfs::<I>`, `Matrix`, and `@memo`. Because `(Row, Col)` is not accepted as a single `Index`, **7 of 10 ports in Batch 12** had to fight grid cells and **4 of 10** (`ccc18s3`, `coci10c7p4`, `wossoly21s3`, `dmopc21c6p3`) had to hand-write BFS instead of calling `std.graph.bfs` (`L5`), while `noi20p1` manually packed `(City, 0..5)` into `c.to_idx * 5 + wait` (`N15`). Because `Bool`, fieldless enums, payload enums of domains, and `[D]E` arrays do not derive `Index`, flow/2-SAT/layered/circuit graphs manually pack offsets (`K11`, `9-T8`, `tle17c5p6`), ternary-prime and board-game DPs manually pack base-3 integers (`P5`, `L23`), and two `[0..d]Z` or `[n]0..=1e9` range tables fail to unify with an `"expected X, got X"` error (`4-B9`, `H21`, `R4`).

### Rank 3. Ordered containers and unambiguous binary vs linear search (`F10`, `F13`, `2-B1`, `K6`, `S13`, `S14`, `D1`, `D8`, `H19`, `9-T12`, `10-T8`, `N`-seen-again, `L6`, `L13`, `R1`, `R2`, `R9`, `R10`, `P13`)
- **Reach:** 13 of 14 batches; ~100+ corpus problems (48+ ordered-bag/multiset/PBDS including `noi18p4` + 47–51 hand-written `while (lo < hi)` binary search + linear `find`/`argmin`/`min_by`).
- **What happens today:** Missing ordered lookups (`first_ge`, `last_lt`, `rank`, `nth`) forced `noi04p1` (`D1`) and `etsr23r1p2` (`S13`) offline, cost `noip18p3` 24 lines of skip-list boilerplate (`2-B1`), and forced `noi18p4` to O(n^2) list rebuilding. Worse, Batch 9 (`9-T12`) revealed a **dangerous API trap**: `first_where` is a *binary search* under a linear-search name, so `ioi19p2` called `Spot.first_where(...)` on a non-monotone predicate and got a **silent wrong answer** — even while Batch 13 (`R1`, `R2`) still needed `last_*`, `first_some` (returning the witness so `parks(reach)` is not run twice), `r.mid`, `bisect` on `F64`, and `local_max`. Meanwhile genuine linear search (`a.find(x)`, `a.where(pred)`, `first(...)`, `min_by`/`argmin` — `L6`, `L13`, `9-T12`, `P13`) and binary search over `List` (`D8`, `S13`, `H19`, `9-T18`) are missing.

### Rank 4. Uniform slicing, sequences, grids, and sliding windows (`G2`, `G5`, `G21`, `K10`, `K14`, `K15`, `K16`, `S6`, `S12`, `S15`–`S19`, `D7`, `D10`, `H8`, `H10`, `H15`, `H18`, `H20`, `10-T3`, `10-T4`, `N8`, `L1`–`L4`, `L8`, `L12`, `L15`, `L16`, `L18`, `L22`, `R3`, `R8`, `P6`, `P14`)
- **Reach:** All 14 batches; every CP domain (143 corpus `[i + 1]` files, 102 `dx`/`dy` grid files, 65 `deque` files, 45 `reverse` files, 30 `unique` files, 16 `j += i` step loops).
- **What happens today:** Arrays slice, `List` and `Str` do not (`H10`, `H20`, `S19`); `std.strings` requires `List<T>` so 6 of 10 Batch-10 ports copy strings/arrays first (`10-T3`); `.len` vs `.len()`, `.last` vs `.last()` (non-mutable), missing `.is_empty` and `.dedup()` on `List` (`S6`, `K9`, `P14`), missing `(lo..=hi).step(k)` (`N8`), missing `l.r..r.l` boundary ranges and `b - a: Int` same-domain distance (`P6`), missing `Dir4`/`step`/`around`/`ray`/`flipped`/`transposed` (`L1`–`L3`, `L22`), missing `Deque` (`2-B11`, `S12`, `L4`), and repeated manual sliding-window / monotone-deque / two-pointer / Mo loops (`K14`, `D7`, `L8`, `L15`, `L16`, `R3` — notably, TWO independent scout sessions both introduced the exact same empty-window bug `hi < lo` when hand-writing `coci10c7p4`'s two-pointer loop!).

### Rank 5. Input/output shapes, dependent bounds, graders, checkers, and stress generation (`F3`, `F20`, `F21`, `F25`, `F26`, `G4`, `G8`, `G14`, `G25`, `4-B13`, `K2`–`K4`, `K19`–`K21`, `S1`, `S4`, `S5`, `S8`, `D11`, `H1`, `H9`, `H16`, `9-T16`, `9-T17`, `N14`, `N16`, `L20`, `L26`, `L27`, `R5`, `R10`, `P3`, `P14`)
- **Reach:** All 14 batches; 106 IOI grader problems, 60 command-dispatch problems, 44 sentinel-`while` input problems, 31 interactive problems, 9 of 10 Batch-9 ports and 5 of 7 Batch-13 ports writing custom generators for standard promises (`Tree`, `Sorted`, `Perm`, connected graphs).
- **What happens today:** Tuple reads `let (n, k): (1..=2e5, 1..=n) = read()` fail (`S8`, `N14`), mutable list reads `var xs: List<Int> = read(count: m)` fail (`N16`), `ask` fails outside a whole `let` (`R10`), record-field dependent bounds are ignored (`S4`, `K19`, `R`-seen-again), `?T` cannot read sentinel `none` (`read(none: 0)` / `read(none: '.')`, `L20`), enums without `@tags` silently read variant 0 (`L26`), raw text (`read_line()`, `read_chars(n)`, `D11`) and header-prefixed `@multitest` (`P14`) are missing, string interpolation `"{x:.2}"`/`"{n:04}"` is missing (`G4`, `10-T10`), and co-located `checker` / `interactor` / `@grader` blocks are not wired up yet (`9-T17`, `L27`, `F20`).

### Rank 6. Algebraic data structures, linear algebra, math, DP opts, trees, graphs, bitsets, and strings (`F2`, `F7`, `F16`, `F19`, `2-B2`–`2-B8`, `2-B12`, `4-B8`, `4-B14`, `G7`, `G12`, `G13`, `G16`, `K8`, `K9`, `K13`, `S11`, `S18`, `D2`, `D6`, `D9`, `D12`, `H4`–`H6`, `H21`, `9-T1`–`9-T7`, `9-T9`–`9-T11`, `9-T14`, `10-T1`, `10-T4`–`10-T7`, `10-T9`, `10-T12`, `N1`–`N4`, `N9`–`N12`, `L7`, `L11`, `L19`, `R6`–`R8`, `P10`–`P12`)
- **Reach:** All 14 batches; trees, graphs, DP, strings, range queries, bitsets, linear algebra (44 binary-power files, 10 matrix-mul files), number theory, D&C/Knuth DP.
- **What happens today:** `stdlib/` has clean foundations, and Batch 1 added `Steps`, `scan`, `SegTree.of`, and `Rooted.weighted`. The remaining high-leverage stdlib pieces are: `Rooted` DFS spanning tree + `up_arc`/`far`/`top_down`/`bottom_up`/`from_parents`/`tour`/`centroid`/`reroot` + `Hld` with lazy assign (`2-B2`, `9-T1`–`9-T5`, `9-T10`, `9-T11`); `bridge_blocks`, `TwoSat<D>`, `mst`, `two_colour`, `all_pairs`, `Functional<V>`, `reach`, `bfs(limit:, source)`, and automatic 0-1 BFS (`9-T6`, `9-T7`, `L7`, `L19`, `K8`, `G16`, `2-B12`); generic `compress` over `Ord` (`9-T9`, `D9`); `Semiring<T>` (`MaxPlus`/`MinPlus` over `?T`), `Matrix<R, C, T>`, `Powers` binary-lifting table, `fold_balanced` (`R7`), `std.linalg` (`row_reduce`/`det`/`rank`/`solve`/`XorBasis`), `std.poly` (`Poly<T>`, `poly_at`, `convolve`/NTT), `Sieve.multiplicative` (`mobius`/`phi`/`divisor_count`), `Congruence`, and multi-variable `quotient_blocks` (`N1`–`N4`, `N9`–`N12`, `G12`, `G13`, `9-T14`); `RangeApply`, stock `?Int` lazy actions, 2-D `Fenwick`, `Dsu` with node semigroup / edge group (`2-B3`, `2-B4`, `D2`, `D6`, `D12`, `10-T7`); `dnc_opt` and `knuth` (`R6`), `sos_sub`/`sos_sup`/`mobius_*` + `Subset.shifted(k)`/`add`/`remove`/`toggle` + `Bitset` operators (`P10`, `P11`, `H4`, `H5`, `L11`); `Hash`/`StrHash`/`Hash2D`/`SuffixArray`/SA-IS/`overlaps`/`borders` (`10-T1`, `10-T4`–`10-T6`, `10-T9`, `G7`); and generic `LiChao<F, V>` + `LiChaoArena<D, F, V>` (`P12`).

### Rank 7. Numeric completeness: `F64`, unified `Mod` (`const`/`let` + >3.03e9 + `Int` mixing + `sum`), `U64`/`I128`, bit & char ops (`F15`, `2-B9`, `4-B5`, `4-B10`, `G1`, `G3`, `G9`, `G10`, `G14`, `G15`, `G20`, `K17`, `H2`, `H9`, `H11`, `H12`, `10-T2`, `N5`, `N6`, `L9`, `L25`, `R9`, `P4`, `P11`)
- **Reach:** 11 of 14 batches; 9 blocked float ports (`ccc05s5`, `dpj`, `noip01p4`, `cco04p4`, `etsr23r3p2`, `cco19p1`, `ceoi17p2`, `noi05p4`, `ccc20s5` — 53/710 corpus float files), 6 of 10 Batch-11 ports wrapping `Z(...)` 42 times (`N5`), 4 of 10 hitting broken `sum` over `Mod` (`N6`), 45 wide-integer files, 407 char-test/offset files, 145 bitmask files.
- **What happens today:** Missing `F64` (and float `@memo`, `F64` ranges `1.0..=1000.0`, `bisect`) blocks geometry, probability DP, and real-valued search completely; `Mod<P>` rejects runtime moduli (`2-B9`, `P7`), caps `P` at `3.03e9` (`10-T2`), and diverges between `proto/cpc` and `proto/cpi` on `Mod + Int` mixing and `sum` identity (`N5`, `N6`); `U64`/`I128`, `popcount`/`ctz`/`clz`, `sign`, and `Char.is_lower`/`lower`/`digit` are missing.

### Rank 8. Growable node arenas (`Pool<T>`) and `??=` (`D4`, `D5`, `D3`, `10-T10`, `P12`)
- **Reach:** 3 batches (`cco17p3`, `ioi12p3`, `sac22cc3p5`, `noi03p2`, `noi19p1`), 65 corpus node-arena / trie / treap / persistent-tree / Li Chao forest problems.
- **What happens today:** Dynamic trees and tries drop out of typed indexing into raw `Int` handles and repeat `if nodes[o].kid[b] == none: nodes[o].kid[b] = fresh(); o = nodes[o].kid[b]!` everywhere.

### Rank 9. Control flow, `with` scope guards, and expression uniformity (`F5`, `F8`, `F9`, `F12`, `F23`, `F24`, `4-B12`, `G17`, `G19`, `S6`, `S7`, `S9`, `H14`, `H17`, `9-T13`, `9-T18`, `N11`, `N13`, `L14`, `R10`, `P2`, `P9`)
- **Reach:** 12 of 14 batches, every domain.
- **What happens today:** Missing `match` / `if let` / multiline `if` expressions (`G19`, `S7`, `N11`, `P9`), `for ... if cond:` loop filters (`P9`, `9-T18`), `while let ... and` and mid-`and` `let` (`S6`, `4-B12`), one-line `if` in match arms (`F24`), top-level `return` (`S7`, `9-T13`), `unreachable` (`G17`), nested comprehension `for`s (`S9`, `9-T18`), `var (a, b) = ...` mutability (`P2`, `N13`), `ask(...)` in expression positions (`R10`), `with guard(): body` scope-guard statement over `lends ()` (`L14`), and last-use move (`F23`).

### Rank 10. Disjoint mutation, write-back projections, and small-to-large (`F17`, `F18`, `F27`, `2-B6`, `4-B3`, `S14`, `H7`, `10-T11`, `L17`)
- **Reach:** 7 of 14 batches; small-to-large tree DP, counting, segtree point updates, in-place row writes.
- **What happens today:** `F18` and `take` landed in `proto/cpc`, while `proto/cpi` still traps on `m[k] += 1` (`10-T11`). Small-to-large merging still trips on `law disjoint` (`2-B6`, `H7`, `F27`), `a.row(r) = row` emits invalid C++ (`4-B3`), and `SegTree` lacks write-back `t[i] += 1` (`S14`).

### Rank 11. Prototype bugs causing memory unsafety, silent WAs, or invalid C++ (23 logged; 13 closed by the bug sprint, 10 open)
- **Closed on `main` by `2b25e6a` and `d6cf87d` (`decisions.md` §15), each locked by a test under `crates/cpl-cli/tests/cases/`:** `L0` destructuring `let (a, b) = xs[i]` (and `if let`, and a `match` subject) was a dangling C++ reference and is now a copy (15.4; the blind critic hit the same heap-use-after-free independently in `4342005`); `K2` `print` evaluates every argument before it writes (15.1); `H17` a range's ends are read once (15.2); `K5` a large `@memo` product is hashed past 2^26 slots (15.3); `K1` records of hashable fields are hashable (15.5); `K4` a `gen:` value is written as `read()` reads it (15.6); `H2` `f<P>(x)` is an error that says so (15.7); `S3` `MinHeap<T>()` takes no arguments (15.8); `L26` an enum with no fields and no `@tags` is a compile error that gives the `@tags(...)` line (15.9); and `S1`, `S2`, `S5`.
- **Still open, silent wrong answers first:** `9-T12` `first_where` is a binary search under a linear-search name; `P3` a `read()` inside a top-level `while` swallows the rest of the program; `4-B2` `Subset - Subset` is an `Int`; `G8` `[Row, Col]Bit` is read as numbers; record-field dependent bounds are ignored (`S4`; `R`-seen-again). Then invalid C++ and crashes: `P1` `using namespace std;` collides with `norm`/`next`/`count`/`rank`; `P2`/`N13` `var (a, b) = ...` binds immutable names; `N7` `[D](lo..=hi)` is read as a one-field record; `G25` and the blind critic's finding 3, the same root: every user `struct` gets a synthesized `operator<`, so a struct holding an enum, a `?Int` or a `List<?Int>` fails in `g++`; a user `struct SegTree` or `struct Dsu` crashes `proto/cpc` with a Python `IndexError` (blind critic 3); `4-B3` `a.row(r) = row`; `G1` `gcd` of positive ranges "may be 0"; `9-T18` `1 << 30` "may be 0". Left for this document by 15.7: how a type argument is written, `f::<P>(x)` (§6, row 2.13).

---

### 1.1 What the blind critic changes in this ranking

`research/blind-critic-audit-1.md` is a different kind of evidence from the scout batches: eight unseen problems written from the user-facing docs alone, under simulated contest time, with the compiler source closed. Every one of the eight first drafts failed its first compile. It confirms the ranking above and sharpens three points.

**`Str` is opaque, and this is a design finding, not a prototype gap.** Thirteen blind probes on a `Str` failed: `s[0]`, `for ch in s`, `s[i..j]`, `s + t`, `.copy()`, `.join(...)`. A word whose letters can be read exists only as `let w: [domain W]Char = read()` on a standalone `let`, so a dictionary `[D](c: Char, x: Str)` cannot be read at all and the string DP `4331904` came out 35% longer than its library-stripped C++. Batch 8 `H10` and batch 10 `10-T3` reported the same wall from the scout side. `decisions.md` 1.6 already says a `Str` is `[domain]Char`, a package that opens where it is bound; what the critic shows is that the opening must happen wherever a `Str` is stored (a record field, an array element, a list element), or the type is a trap. §2.3 B states the rule, and it moves `Str` from Rank 4 detail to a Phase 3 gate.

**The docs are part of the language.** Six of the eight first-compile failures were syntax the docs show and `proto/cpc` rejects: tuple `read()`, `first_true` against `first_where`, `if c then a else b` against `if c: a else: b`, positional enum payloads and `case` arms, `Deque`/`HashSet`, `mut fn`. A contestant cannot tell a design gap from doc drift, and each costs two to five compile cycles. This is Rank 1 seen from outside: one compiler, and one cheat sheet whose every snippet is compiled in CI (§7 Phase 1).

**A user's own struct must never collide with the library's.** Writing `struct SegTree` because the docs do not list the library's constructor crashed the compiler, and renaming it failed in `g++`. Both are bugs (Rank 11), and the cause is the stand-in design of Rank 1: names hardwired into the compiler.

The critic also re-found, independently, the ordered `Map`/`Set` gap (Rank 3; it forced an algorithm change on `3540475`), `popcount` and `>>=` (Rank 7), map bucket initialisation and `max` over `?Int` as a `SegTree` operation (Ranks 9 and 6), and the small-to-large borrow rejection with a help line that names `self` where it means `self.comps[u]` (Rank 10). What it found pleasant matches the scouts' "Already good" lists: boundary domains, `?Int` with `min=`, `Mod<P>`, exhaustive `match`, and the compile-time warning that a loop nest can run 4e10 times.

---

## 2. Unified Language & Stdlib Architecture

The guiding test from `decisions.md` is **general before special**: every recurring friction cluster should collapse into an existing kernel mechanism or a small userland module, never a bespoke compiler rule. Below is the smallest set of orthogonal mechanisms that absorbs all eleven clusters.

### 2.1 One Unified Model for `Index`, `Range`, `Subset`, `Pool`, Tree Paths, `Mod`, and `Float`

`plan/19-index-and-lends.md` already reduced `domain`, `Str`, `Grid`, and `Perm` to `Index`, `Seq`, fresh index binders, and `lends`. Completing those exact same ideas — with zero grid-specific, graph-specific, or DP-specific compiler rules — brings every remaining index-like and numeric type into one clean model.

#### A. Complete algebraic `Index` derivation (scalars, `?T`, products, sums, exponentials)

Every finite type constructor in CP Lang derives `Index` automatically under one uniform algebraic rule:

- **Finite scalars and `?T`:** `Bool` (`K = 2`), any fieldless `enum` (`K = variant count`, `4-B11`, `P5`, `L26`), any bounded integer range `lo..hi` / `lo..=hi` (`K = max(0, hi - lo)`, `4-B1`, `4-B9`, `4-B11`), and `?T` whenever `T: Index` (`K = T.K + 1`, `L23`). Ranges and domains also implement `Seq` (ordered line positions with `.l`/`.r`, `b - a -> Int`, `+`/`-` optional steps, `.cyc`). Two anonymous range dimensions with identical written bounds in scope (`[0..d]Z`) are the same type (`4-B9`, `H21`).
- **Products (`A * B`):** Any tuple `(A, B, ...)` or record `(city: City, corner: Corner)` whose fields all implement `Index` derives `Index` automatically in row-major order with `K = prod(field.K)` (`G18`, `L5`). Crucially, a product of domains works **everywhere** an `Index` is expected: `for p in Cell`, `[Cell]Bool`, `Dsu<Cell>`, `bfs::<Cell>`, `SegTree<Cell, T>`, and `@memo` (`L5`, unblocking 7 of 10 Batch-12 grid ports).
- **Sums (`A + B`):** Any payload `enum` whose variants are unit or carry `Index` payloads derives `Index` automatically with variants laid out end-to-end, `K = sum(variant.K)` (`K11`, `9-T8`). No separate `domain Node = A | B` syntax is needed: an `enum` already IS a sum type!
- **Exponentials (`E^D`, i.e., `[D]E`):** Any fixed-domain array `[D]E` where `D: Index` and `E: Index` derives `Index` automatically via radix encoding (`K = E.K ^ D.K`, subject to the existing `2^31 - 1` `Index` cardinality limit, `P5`, `L23`). Under this single rule, `Subset<D>` is simply the bit-packed specialization of `[D]Bool`, ternary prime masks `[Small]Side` (`noi15p3`, `P5`) require zero `pow3[]` math, and whole-board game states `[Cell]?Player` (`cco19p1`, `L23`) index `@memo` directly with zero hand-written base-3 encoding!

Before (`noi06p4` `K11` and `sdoi17r2p4` `9-T8` — manual offset packing):
```cp
domain Node = 0..n + m + 2
let source = Node(n + m)
let sink = Node(n + m + 1)
fn station(s: Station) -> Node: Node(s.label - 1)
fn user(u: User) -> Node: Node(n + u.label - 1)
net.add(user(u), station(users[u].a), inf)
```

After (derived sum `Index` — ordinary `enum`, zero arithmetic, impossible to collide):
```cp
enum Node:
    Source
    Sink
    Build(s: Station)
    Serve(u: User)

var net = FlowGraph<Node>()
net.add(Serve(u), Build(users[u].a), inf)
```

Before (`noi15p3`, `P5` — manual base-3 digit extraction):
```cp
let pow3 = [1, 3, 9, 27, 81, 243, 729, 2187, 6561]
for m3 in 0..total3:
    let digit = (m3 / pow3[prime_id[c] - 1]) % 3
```

After (derived exponential `Index` on `[Small]Side` — zero `pow3` math):
```cp
enum Side = Neither | A | B
type Assign = [Small]Side                   // Index automatically: K = 3^8 = 6561
var ways: [Assign]Z = 0
for assign in Assign:
    if assign[p] != A: ...
```

Before (`wossoly21s3`, `L5` — 14-line hand-written grid BFS because `(Row, Col)` was not an `Index`):
```cp
var dist: [Row, Col]?Int = none
var todo: List<Cell> = [from]
var next = 0
while next < todo.len:
    let (r, c) = todo[next]                 // also hit the L0 use-after-free bug!
    ...
```

After (`(Row, Col)` is `Index` everywhere + `std.grid.around`):
```cp
type Cell = (Row, Col)
fn walk(from: Cell, to: Cell) -> ?Int:
    bfs::<Cell>(p => (q for q in p.around() if g[q] != '#'), from: [from])[to]
```

**Unified `@memo` layout falls out with zero special cases.** Whenever a `@memo` function's parameter tuple `P` implements `Index` (including `Bool`, enums, ranges, `(Row, Col)`, `[Cell]?Player`, `Subset<D>`) and the table has at most 2^26 slots at the domains' declared maximum sizes (the threshold decision 15.3 landed), `@memo` allocates one flat `[P]?R` array (niche-packed for `Int`, `F64`, domains, enums). Otherwise (like `cco18p4`'s `6e6 x 4e5` interactive grid `K5`, or `Int`/`Str` parameters), `@memo` uses `HashMap<P, R>`. `@memo(flat)` and `@memo(hash)` override explicitly, and `cpl explain` / hover shows the chosen layout.

#### B. `Subset<D>`, `Tri<D>`, and boundary subranges (`4-B2`, `4-B6`, `H13`, `P6`, `P10`, `P11`, `L11`, `F4`)
- **`Subset<D>`** is an `Index` type (`0..(1 << D.K)`). Pure userland impls provide set algebra `|`, `&`, `-` (set difference, fixing `4-B2`), `^` (symmetric difference, `H13`), `<=`, `in`, `.subsets`, `.single(x)`, `.bits`, mutating `.add(x)`/`.remove(x)`/`.toggle(x)` (`L11`), and `.shifted(k)` mirroring `Bitset<D>.shifted(k)` (`P11`, removing raw bit-twiddling from broken-profile DP `noi23p2`). On top of `Subset<D>`, `std.subset` provides the four 5-line generic Zeta/Möbius transforms parameterized by `Monoid`/`Group`: `sos_sub`, `sos_sup`, `mobius_sub`, `mobius_sup` (`P10`).
- **Boundary subranges and same-domain distance (`P6`):** `plan/19` already approved `v.l` and `v.r`. Making `l.r..r.l` (strictly between `l` and `r`, empty when `l + 1 >= r` with no `?Pos` unwrapping!) work in range literals and `for` loops, alongside `b - a -> Int` for two positions of one `Seq` domain, removes all `.idx` escapes from interval DP (`csps21p2`, `dmopc18c1p6`) and HLD tail ranges (`9-T3`).
- **Triangular tables (`F4`):** A 10-line userland `struct Tri<D>` implementing `Index` (`K = D.K * (D.K + 1) / 2`, `to_idx = r.to_idx * (r.to_idx + 1) / 2 + l.to_idx`) and `Read` covers `ccc19s5`, `coci20c6p3`, and triangular interval DP without compiler `where`-clause inversion.

#### C. `Pool<T>` and `p.freeze()`: unifying growable arenas with fresh domains (`D4`, `10-T10`, `P12`, `plan/13 M9`)
Tries, persistent segment trees, implicit treaps, suffix automata, and multi-tree Li Chao forests (`cco17p3`, `ioi12p3`, `sac22cc3p5`, `noi03p2`, `noi19p1` — 65 corpus problems) allocate nodes dynamically. Instead of adding a `pool Nodes: Node` keyword (`plan/13 M9`), `Pool<T>` is a library struct reusing the **exact same fresh-type binder that `[domain]T` already uses for `s.Idx`** (`plan/19 §3.2`, §4.4):

- `var nodes = Pool<Node>()` introduces a fresh, sealed handle type `nodes.Id` scoped to the block (cannot escape or mix with another pool or `Int`).
- `nodes.Id` is a **monotone handle** (`trait Handle`: sealed `ord: U32`, `Niche` so `?nodes.Id` is 4 bytes, `Copy`, `Eq`, `Ord`, `Hash`), NOT a fixed-size `Index`.
- Because `Pool<T>` is append-only (or tracks generation/free-list in debug mode when created with `Pool.reusable()`), `nodes.push(x) -> nodes.Id` maintains `h.ord < nodes.len`, so `nodes[h]` is **bounds-check-free** just like `[V]T`.
- Side tables while growing use `PoolMap<nodes.Id, V>` (auto-growing `List<V>` indexed check-free by `nodes.Id`).
- Once construction finishes, `let (Node, frozen) = nodes.freeze()` consumes `nodes` and returns an ordinary `Index` domain `Node` (`let K = nodes.len`) plus `frozen: [Node]T`, immediately unlocking `[Node]T`, `Rooted<Node>`, `Dsu<Node>`, and Aho-Corasick failure links (`10-T10`).

Before (`cco17p3`, `D4` + `D5` — raw `Int` indices and triple-written optional slot):
```cp
fn reach(root: Int, x: Int) -> Int:
    var (o, rest) = (root, x)
    while rest > 1:
        if nodes[o].kid[rest % 2] == none: nodes[o].kid[rest % 2] = fresh()
        o = nodes[o].kid[rest % 2]!
        rest /= 2
    o
```

After (`Pool<Node<_>>` supplying the fresh `nodes.Id` handle parameter + `Bool: Index` + `??=`; note: whether the self-handle is supplied via `Pool<Node<_>>` or a `Node` type parameter bound in `Pool.new` is a small syntax detail to verify in Phase 3):
```cp
struct Node<Id>:
    kid: [Bool]?Id = none
    total: Int = 0
    ys: ?Id = none

var nodes = Pool<Node<_>>()
fn reach(root: nodes.Id, x: Int) -> nodes.Id:
    var (o, rest) = (root, x)
    while rest > 1:
        o = nodes[o].kid[rest % 2 == 1] ??= nodes.push(Node())
        rest /= 2
    o
```

#### D. Tree paths and decompositions fit `Index` + `Range` (`2-B2`, `4-B14`, `H21`, `F16`, `9-T1`–`9-T6`, `9-T10`, `9-T11`)
`Rooted<V, A = ()>` in `stdlib/tree.cp` already maps subtrees to `Range<Pre>` and paths to O(log n) `Range<Pre>`s. Pure-library completions remove every tree friction item across Batches 1, 2, 4, 8, and 9:
- Build `Rooted` via **DFS** (not BFS, fixing the `stdlib/tree.cp` discrepancy noted in `decisions.md` §12 and `9-T6`) so non-tree edges on general graphs are strictly back-edges.
- Retain the edge/arc payload `A`: `tree.up_arc(v) -> ?A` (and `tree.far(v)` for weighted root distance, `9-T4`, `2-B2`, `4-B14`).
- Expose `tree.pos(v) -> tree.Pre` (typed as `Pre`, never raw `Int`!), `tree.at(p: tree.Pre) -> V`, `tree.parent(v)`, `tree.children(v)`, `tree.depth(v)`, `tree.size(v)`, zero-allocation `tree.top_down` / `tree.bottom_up` generators, `tree.tour` yielding `(v, entering: Bool)` (`9-T10`), `tree.centroid` / `tree.centroids` (`9-T11`), `tree.reroot(monoid, lift:, undo?:)` (`2-B8`, `9-T5`), and `Rooted.from_parents(parent)` (`2-B2`, `H21`).
- Expose `Hld<V, T>` (already in `stdlib/tree.cp`) with `path(a, b)`, `path_edges(a, b)`, `sub(v)`, and an optional lazy `Action` variant (`LazyHld<V, T, F>` or `Hld.lazy(tree, m, act, init)` covering `noi21p1`'s ordered path assignment, `9-T1`, `9-T2`).
- Add `bridge_blocks(g) -> (domain B, [V]B)` and `TwoSat<D>` in `stdlib/graph.cp` using the exact same fresh-domain pattern as `scc` (`9-T6`, `9-T7`).

#### E. Unifying `Mod<P>` and `ModR` via frozen `let` (`2-B9`, `10-T2`, `G14`, `H9`, `H11`, `P7`)
`decisions.md` 3.10 split `Mod<P>` (compile-time constant) and `ModR` (`r.add(a, b)`). Batches 2, 10, and 14 (`dpv`, `ctsc06p1`, `noi15p3`) showed that nobody uses `r.mul(a, b)` in practice — they drop back to manual `% m`. Meanwhile, `plan/19 §3.1` ALREADY introduced `let K = n` (an associated runtime-frozen value on a trait impl). Applying the exact same mechanism to `Mod`:

```cp
pub trait Modulus:
    P: Int                                  // `const P = 1_000_000_007` OR `let P = m`

pub struct Mod<M: Modulus>(v: Int): ...
impl Mul for Mod<M>:
    fn mul(a, b):
        if const M.P <= 3_037_000_499: Mod(a.v * b.v % M.P)
        else: Mod(mul_mod(a.v, b.v, M.P))   // unlocks Mod<2^61 - 1> (10-T2) at zero cost for 1e9+7
```

Now `type Z = Mod<1_000_000_007>` and `type Z = Mod<m>` (for `let m: 1..=1e9 = read()`) are **the same `Mod` type** — identical `+ - * / **` operators, identical `Comb<M>`, `prefix`, `Fenwick`, `SegTree`, `Matrix`, zero extra bytes per element, and streaming arbitrary-length decimal `impl Read` (`G14`, `H9`). Lock in two numeric rules consistently across both compiler and interpreter (`N5`, `N6`): (1) an `Int` next to a `Mod<M>` in `+ - * /` and assignment reduces into `Mod<M>` (one-way; a `Mod<M>` never silently turns back into `Int` without `.v`), and (2) `sum` and `product` start from `T.zero` and `T.one` of the element type so `sum(a[i, j] * b[j, k] for j in Off)` works directly over `Mod`. `ModR` as a separate `r.add(a, b)` API is retired.

Before (`dpv` `2-B9` and `noi15p3` `P7` — scattered `% p` and `1 % p` guards):
```cp
let p: 1..=1e9 = read()
var below: [V]Int = 1 % p
for c in tree.children(v): below[v] = below[v] * (1 + below[c]) % p
```

After (one `Mod` type for compile-time and runtime moduli):
```cp
let p: 1..=1e9 = read()
type Z = Mod<p>
var below: [V]Z = 1
for c in tree.children(v): below[v] *= 1 + below[c]
```

#### F. Settling `Float` (`F64`) and `U64`/`I128` (`F15`, `4-B5`, `K17`, `G4`, `G9`, `G15`, `L25`, `P4`)
Eight scout ports across five batches (`ccc05s5`, `dpj`, `noip01p4`, `cco04p4`, `etsr23r3p2`, `cco19p1`, `noi05p4`, `ccc20s5`) are blocked solely on missing floats (`decisions.md` 1.5):
- **One standard float type: `F64`** (IEEE-754 double, 8 bytes; we can alias `Float = F64` or pick `F64` consistently across docs and prelude — pick one canonical name `F64` so there is never a `Float`-vs-`F64` guess).
- **Quiet-NaN niche:** Just as `Int` reserves `LLONG_MIN` so `?Int` is 8 bytes, `F64` disallows NaN as a valid value (creating NaN via `0.0 / 0.0` or `sqrt(-1.0)` traps in local debug builds with the source line), making canonical quiet NaN the `?F64` niche (`sizeof(?F64) == 8`!). `F64` implements `Ord`, `Niche`, `Copy`, `Relocate`, `Zero`, so `@memo` returning `F64` (`P4`, `4-B5`, `L25`) uses a flat `[Params]?F64` table with zero extra `seen` array!
- **Explicit variable conversion, unified literals:** `Int` variables require `F64(x)` (preventing accidental Euclidean-vs-float `/` confusion and silent precision loss past 2^53), while integer literals (`0`, `1`) unify with `F64` in typed positions and `if` branches (`4-B10`).
- **Equality and output:** `==` on `F64` compiles with a lint suggesting `close(a, b, eps: 1e-9)`. `print(x)` defaults to judge-safe precision (`12` significant digits), while `print(x, digits: 3)` and string interpolation `"{x:.3}"` format fixed decimals (`G4`, `P4`).
- **Generic `Pt<T>` in `std.geometry`:** `Pt<Int>` for exact integer geometry (`cross`, `dot`, `hull`, `area2`, `norm2`); `Pt<F64>` adds `dist`, `reflect`, `project` (`G15`, `K17`).
- **Fixed-width integers & bits (`1.7`, `1.8`, `G9`, `H12`, `P11`):** Confirm `U64` (wrapping, full 64-bit shifts) and explicit `I128`; expose `.popcount`, `.ctz`, `.clz`, `.bit_len`, and `bit_ceil` on both `Int` and `U64`.

---

### 2.2 Evaluating `slide(k, add:, drop:)` + the `Slide` Trait

Across all 14 batches, four distinct window patterns recur:
1. **Fixed-width sliding window** (`ccc20s3` letter counts, `dpm` modular sum window, `ioi06p2` `K14` 2-D sliding min, `noi05p1` `L15` monotone-queue DP).
2. **Variable-width monotone two-pointer window** (`noi16p4` shortest interval window with cover depth >= m, `coci10c7p4` and `ioi05p1` `L8`/`R3` shortest valid stretch, `ioi11p3` `R3` longest valid prefix-sum window without mutable state, `plan/12 §6.5` longest valid window).
3. **Arbitrary-order window movement — Mo's algorithm** (`coci20c6p5` `D7`).
4. **Two-pointer draining of two sorted sequences** (`dmopc21c8p1`, `dmopc20c4p3` `S12`) and **BFS worklists** (`2-B11`, `L4`).

**Architectural verdict:** Yes — a two-level **`Window<I>` / `BagWindow<I>`** trait (the `Slide` protocol) plus inline-callback overloads `slide(k, add:, drop:)` cleanly generalizes patterns (1), (2), and (3) in pure userland with zero new compiler constructs. Crucially, however:
- It MUST distinguish **FIFO queue-ordered drops** (`Window<I>`) from **arbitrary-order drops** (`BagWindow<I>`), because `MonoDeque<T>` (sliding min/max, `K14`, `L15`) and `TwoStackFold<M>` (sliding `gcd`/matrix over any `Monoid`) only support FIFO drops and are algorithmically invalid for Mo's algorithm (`D7`), whereas `Counter<K>`, `GroupWindow<G>`, `SortedBag<T>`, and custom Mo structs support adding and dropping at either end.
- It must **NOT** try to absorb pattern (4). Draining two separate sorted lists (`S12`) and BFS queues (`2-B11`, `L4`) belong to **`Deque<T>`** with `pop_front_while(ok)` / `pop_back_while(ok)`.

```cp
// FIFO window (oldest active position drops first).
// Implemented by MonoDeque, TwoStackFold, Counter, GroupWindow, SortedBag.
pub trait Window<I>:
    type Out
    fn add(mut self, i: I)
    fn drop(mut self, i: I)
    fn value(self) -> Out

// Unordered/invertible window (elements may enter or leave at either end).
// Implemented by Counter, GroupWindow, SortedBag, user Mo states — NOT MonoDeque.
pub trait BagWindow<I>: Window<I>
```

Three userland drivers in `std.window` sit on top of this pair of traits:
1. **Fixed-width `D.slide(k, w)`** (and inline-callback `D.slide(k, add:, drop:)`): yields `(win: Range1<D>, val: W.Out)` for every `k`-wide window. On a 2-D table, `g.sliding(Min<Int>, rows:, cols:)` (`K14`) is 8 lines of userland calling `slide(k, MonoDeque(...))` along rows and then along columns!
2. **Variable-width monotone `D.slide_while(w?, ok:)` and `D.slide_until(w?, ok:)`** (plus inline-callback overloads, with `w` / `add:` / `drop:` optional so stateless O(1) prefix-sum windows like `ioi11p3` `R3` pass only `ok: w => cost(w) <= budget` over guaranteed non-empty `Range1` windows!): `slide_while` keeps the longest valid window ending at each `r` (`plan/12 §6.5`, `ioi11p3`); `slide_until` keeps the shortest valid window starting at each `l` (`noi16p4`, `coci10c7p4`, `ioi05p1` — eliminating the `hi < lo` empty-window bug that TWO independent sessions hit in `L8`/`R3`!). Both check monotonicity in debug runs.
3. **Mo's algorithm `mo(queries: [Q]Range<D>, w)`** (requires `W: BagWindow<D>`, plus inline-callback `mo(queries, add:, drop:)`): orders queries in serpentine/Hilbert block order, always grows before shrinking (`D7`), and statically prevents passing a FIFO-only `MonoDeque`.

*(Borrow-checker note on inline callbacks `add:` / `drop:`: Under `plan/19 §5.3`, an `inline` callback is checked in the caller's body. Because `slide` and `mo` invoke `add` and `drop` at separate sequential statements — never nested inside each other — two `inline` callbacks mutating the same caller `var` do NOT alias simultaneously. Confirming sequential non-overlapping borrows for `inline` parameters in decision `5.4` lets `noi16p4` and `coci20c6p5` pass two 1-line closures without defining a wrapper struct or threading state manually!)*

Before (`noi05p1`, `L15` + `L16` — 12 lines of manual monotone queue + obstacle reset):
```cp
for p in line:
    if hall[p] == 'x': reach = []; head = 0
    else:
        while head < reach.len and reach[head].pos < i - seconds: head += 1
        while reach.len > head and reach.last.val <= val: reach.pop()
        reach.push((i, val))
        best[p] = reach[head].val + i
```

After (`split` + `slide` over `MonoDeque`):
```cp
for run in line.split(p => hall[p] == 'x'):
    for (p, best_prev) in run.slide(seconds + 1, MonoDeque(Greater, p => far[p]? - p.to_idx)):
        best[p] max= best_prev + p.to_idx
```

Before (`coci10c7p4`, `L8` — manual two-pointer loop):
```cp
var top = 0
for floor in 0..levels.len:
    while top < levels.len and not delivers(levels[floor], levels[top]): top += 1
    if top == levels.len: break
    best = min(best, levels[top] - levels[floor])
```

After (`slide_until`):
```cp
for w in levels.slide_until(until: w => delivers(levels[w.first!], levels[w.last!])):
    best min= levels[w.last!] - levels[w.first!]
```

---

### 2.3 Uniform Sequences, Unambiguous Search, Ordered Containers, Grids, and Write-Back Projections

#### A. Fixing the `first_where` trap (`9-T12`, `F10`, `K6`, `D8`, `S13`, `H19`, `L6`, `L13`, `R1`, `R2`, `R9`, `R10`, `P13`)
Batch 9 (`9-T12`) proved that naming binary search `first_where` is a live footgun: `ioi19p2` called `Spot.first_where(...)` expecting a linear scan and got a silent wrong answer. Batch 13 (`R1`, `R2`) still liked `first_where` because it had not hit `9-T12`'s bug, so we adjudicate the naming conflict explicitly in favor of safety (`G16`, "hard to write wrong"). Separate binary and linear search cleanly by name across every `Range<I>` and `Indexed<I>` (`Array`, `Slice`, `List`):

- **Binary / local search (monotone or unimodal predicate, O(log n), monotonicity checked at probed points in debug builds):**
  - `r.mid` — midpoint of any `Range<I>` (`R1`, removing `Field((l.label + r.label) / 2)`).
  - `r.partition_point(ok)` — boundary where `ok` switches from `true` to `false` (or `Int` on `List`).
  - `r.bsearch_first(ok) -> ?I`, `r.bsearch_last(ok) -> ?I`, and witness-preserving **`r.bsearch_first_some(f: I -> ?W) -> ?(I, W)`** / `bsearch_last_some` (`R1`, `R2`: `bsearch_first` is just `bsearch_first_some` with `()` payload, eliminating recomputing `parks(reach)` twice and letting `f` short-circuit early!). Accepts lambdas, plain function names, and `.field` (`K6`, `S16`).
  - `bisect(lo..=hi, ok, steps: 200)` for `F64` ranges (`R9`).
  - `xs.first_ge(x, by:)` and `xs.last_le(x, by:)` on sorted `Indexed<I>`, returning `?I` (the position, so `rows[r].push(t)` works in `cco04p2`, `S13`).
  - `argmin_convex(r, f)` (`K7`) and `local_max(r, f)` / `local_min(r, f)` (`R10`, golden-section ~1.44 log2 n probes) in `std.search` with query counts documented.
- **Linear search (arbitrary predicate, O(n)):**
  - `first(it)` — first element of any generator (`first(v for v in Spot if ...)` -> `?Spot`, `9-T12`).
  - `a.find(x) -> ?I` and `a.where(pred) yields I` on any `Indexed<I>` (`L6`, `ccc18s3`, `coci10c7p4`, `wossoly21s3`).
  - `min_by(it, key)` / `max_by(it, key)` (and `argmin` / `argmax` over `Indexed<I>`) returning `?T` (or `?(value, item)` / position, `L13`, `P13`, `9-T12`).

#### B. One sequence, slicing, and grid contract (`L1`–`L3`, `L12`, `L16`, `L18`, `L22`, `10-T3`, `H20`, `S6`, `G2`, `P14`)
- **Uniform `Indexed<I>` & `List<T>`:** `List<T>` supports slicing `xs[l..r]` / `xs[..cut]` / `xs[cut..]` (`H20`, `S19`), `.splice(range, items)` (`D3`), `.is_empty`, `.dedup()` (`P14`), and `xs + ys` / `xs.extend(ys)` (`H18`). `.len`, `.is_empty`, `.first`, and `.last` are `@get` properties everywhere (`K9`, `S6`, `H9`), with `.first`/`.last` as `lends mut` projections on `IndexedMut` (`cover.last!.r max= s.r`, `S6`).
- **Views vs mutation:** `.rev` is always a read-only reversed view; `.reverse()` mutates in place (`G2`, `K10`). `xs.items` yields `(index, elem)` pairs (`H8`, `G11`, `H16`). `xs.adjacent` yields consecutive pairs, `xs.cyc.adjacent` wraps around (`S17`, `L18`), `xs.split(pred)` yields runs between separators (`L16`), `(lo..=hi).step(k)` steps any range by `k` (`N8`, 16 corpus sieve/harmonic loops), and `D.chunks(k)` returns `(domain B, [B]Range1<D>)` with `i.chunk(k) -> B` and `r1 & r2` range intersection (unifying `D10` sqrt blocks and `L12` Sudoku 3x3 boxes!).
- **Generic `Iter<T>` and `Indexed<I>` everywhere:** `List(it)`, `Set(it)`, `counts(it)`, `group(it, by:)`, `sorted(it, by:)`, `zip`, `cycle`, `take`, `take_while`, `Graph.undirected(edges)` accept any `Iter<T>` (`G5`, `G21`, `K8`, `S15`, `S16`, `L3`). `std.strings` accepts any `A: Indexed<I, Elem = T>` where `I: Seq` (`10-T3`, `10-T4`, `G6`). `desc(key)` (`Desc<T>`) handles descending tuple keys and `MaxHeap<T>` = `Heap<Desc<T>>` (`K10`, `S19`, `2-B5`, `K9`, `S11`).
- **A `Str` is usable wherever it is stored (`H10`, `10-T3`, `M2`, `M5`, `M6`, blind critic 2).** One rule, already implied by 1.6 and 2.14: a `[domain]T` package opens at every place it is named, not only at a standalone `let`. So `s[i]`, `for c in s`, `s[l..r]`, `s.len`, `s == t[a..b]`, `.copy()` and every sequence view work on a `Str` that is a record field, an array element or a loop variable, and `[D](c: Char, x: Str) = read()` reads a dictionary. `s + t`, `sep.join(parts)`, `"0" * n` and `Str(chars)` build strings; they are `List<Char>` operations and need no string rule. On top of that, userland only: `c.upper`, `c.lower`, `c.digit -> ?Int`, `Char.digit(d)`, `Char` ranges that iterate, and a forty-line `Scanner` over any `[S]Char` (`peek`, `eat(c)`, `take_while(pred) -> Range<S>`, `done`) that returns positions, so a parser can write back into the text (`M6`).
- **`read` for lists, as `Read` arguments and not as new forms (`M4`, `N16`).** `read(k)` for a `List<T>` of a known length, `read()` for a length-prefixed `List<T>`, `read(none: "X")` for a `?T`. The generator follows from the type in each case.
- **Formatting (`M1`, `G4`).** An interpolation hole takes a format after a colon (width, zero padding, digits after the point), and a format the value does not support is a compile error, never ignored (today `"{cents:02}"` prints `0`). `Fixed<D>` is a stdlib struct, an `Int` scaled by `10^D` with `Read` and `Show`, for coordinates and money that must compare exactly; its generator draws the scaled integer, so ties come up in stress.
- **A line continues after a binary operator, after `=`, or inside an open bracket (`M7`).** Core grammar, and the rule every contestant already expects.
- **Pure-userland `std.grid` (`L1`–`L3`, `L22`):** Built directly on `(Row, Col): Index`: `enum Dir4 = N | E | S | W` (with `.cw`, `.ccw`, `.back`, `.delta`, and `E.from("ULDR", ch) -> ?Dir4` via a general fieldless-enum lookup helper `L2`), `Dir8`, `p.step(d) -> ?Cell`, `p.around(dirs = Dir4)`, `p.ray(d)`, `(Row, Col).lines(d)`, and zero-copy `g.flipped` / `g.transposed` `Indexed` views (`L22`).

#### C. One augmented order-statistic tree core (`F13`, `2-B1`, `D1`, `S13`, `S14`, `S18`, `F19`, `10-T12`)
One balanced-tree core in `stdlib/ordered.cp` (storing subtree sizes and accepting dynamic `Order<T>` values) backs ordered `Map<K, V>`, `Set<T>`, and `SortedBag<T>` (`first`, `last`, `pop_first`, `pop_last`, `first_ge`, `first_gt`, `last_le`, `last_lt`, `take_*`, `rank`, `nth`, `nth_last`, `count(range)`), while `HashMap`/`HashSet` remain O(1) hashed. `Steps<K, V>` (with both `.runs(clip)` and `.values(clip)`, `F19`, `10-T12`) and `RangeSet<K>` (`S18`) share one thin interval wrapper on top of `Map`.

#### D. Resolving the write-back contradiction, `with` scope guards, `??=`, and `take`/`absorb`
- **Write-back projections (`S14`, `stdlib/README.md` contradiction):** `stdlib/README.md` lines 263–271 claimed `lends mut` cannot repair invariants after writes, which predated post-`lend` write-back in `plan/19 §3.4` and `ergonomics-audit.md` §1. Clarify `5.12` and update `stdlib/README.md`: post-`lend` write-back lets `Counter`, `SegTree`, and `LazySegTree` implement `IndexedMut<D>` (`standing[at] += 1`, `c[k] -= 1`) cleanly while keeping `self` borrowed mutably until write-back completes!
- **`with guard(): body` over `lends ()` (`L14`):** `plan/19 §3.4` already defines projections whose post-`lend` tail runs when the caller's continuation ends. Adding the statement form `with f(...): body` for `lends ()` projections — and allowing a `lends ()` guard to return without lending when setup fails (like `stepped()` when a robot hits a wall in `cco02p3`) while guaranteeing teardown on `return`/`break`/`continue`/`?` — unifies backtracking undo (`noip09p4`), interactive DFS (`cco02p3`), and `scoped`/`undo` (`decisions.md` 5.8) with zero new borrow mechanism!
  ```cp
  fn placed(r: Line, c: Line, d: Digit) lends ():
      mark(r, c, d, true)
      lend ()
      mark(r, c, d, false)

  with placed(r, c, d): fill(score + d.label * worth(r, c))
  ```
- **`place ??= expr` (`D5`):** Short-circuiting compound assignment on `?T` returning `T`.
- **`take(mut place)` + `.absorb(other)` + `a.swap(i, j)` (`F17`, `2-B6`, `H7`, `F27`):** `above[f].absorb(take(mut above[v]))` eliminates small-to-large `law disjoint` proofs completely; `swap(mut a[i], mut a[j])` errors suggest `a.swap(i, j)`.

#### E. Completing the algebraic stdlib
- **Trees & graphs:** `Rooted` DFS + `up_arc`/`far`/`top_down`/`bottom_up`/`from_parents`/`tour`/`centroid`/`reroot`; `Hld` + `LazyHld`; `reach`, `bfs(limit:, source)`, automatic 0-1 BFS in `shortest` when edge weights have range `0..=1` (`L7`), `mst` (`L19`), `two_colour` (`K8`), `all_pairs` (`G16`), `bridge_blocks` (`9-T6`), `TwoSat<D>` (`9-T7`), `Functional<V>` (`2-B12`), read-only width subtyping (`2-B7`, `decisions.md` 14.1).
- **Range & algebraic structures:** `Semiring<T>` (`MaxPlus<T>` and `MinPlus<T>` over `?T` where `none` is zero, `N2`), `Matrix<R, C, T>` over `[R, C]T` (including anonymous range dimensions `0..r` and product domains `(City, 0..5)`, `N1`, `N15`), `std.linalg` (`row_reduce`, `det`, `rank`, `solve`, `inverse`, `XorBasis`, `N4`), `power(x, k, op)` (over `Semigroup` for `k >= 1` or `Monoid` for `k >= 0`), `Powers(x, op, up_to:)` binary-lifting table (`G13`, `N3`), and `fold_balanced(m, xs)` for associative divide-and-conquer merges (`R7`), `std.dp` (`dnc_opt` and `knuth` over `Tri<D>` with debug monotonicity checks, `R6`), `std.poly` (`Poly<T>`, O(n) `poly_at` over `Mod<M>`, `convolve(a, b, wrap?:)` with NTT fast path, `9-T14`, `N10`), `RangeApply<D, F>(Monoid<F>)` (`2-B3`), type-inferred `LazySegTree<D>.from`, stock `?Int`-identity actions (`AddMax`, `AddMin`, `AddSum`, `AssignSum`, `AssignMax`, `AssignMin`, `Clamp` — `2-B4`, `D12`, commit `40d4e67`), `prefix(a, g)` over any `Group` (including `Sum`, `Xor`, `ProdGroup<Mod<M>>`, plus `Bool` counting — `4-B8`, `F2`, `N12`), 2-D `Fenwick<(X, Y), T>` via `trait FenwickIndex` (`D2`), `Dsu<V, T>` (node semigroup, `10-T7`) + `PotentialDsu<V, G>` (edge group, `D6`), `fold_others` (`2-B8`), `sos_sub`/`sos_sup`/`mobius_*` (`P10`), `LiChao<F, V>` + `LiChaoArena<D, F, V>` over `Pool` (`P12`), `Sieve.spf` + `Sieve.multiplicative(at: (p, e) => ...)` (`mobius`, `phi`, `divisor_count`, `N9`), `Congruence` (`N11`), `Frac` (`G10`), multi-variable `quotient_blocks(ns...)` (`G12`, `N12`), generic `compress` over `Ord` (`9-T9`, `D9`), `RangeKth` (`10-T8`), `Rope<T>` (`D3`), and `merge`/`diffs`/`zip_longest` in `std.seq` (making convex min-plus convolution a 1-line composition, `R8`).
- **Strings, bitsets, permutations:** Move `Hash`/`StrHash` (`10-T1`) + `Hash2D` (`G7`) to `stdlib/hashing.cp`; upgrade `suffix_array` to SA-IS (`10-T6`) and add `SuffixArray` (`10-T4`), `overlaps` (`10-T5`), `borders` (`10-T9`), `Trie<A>` over `Pool` (`10-T10`); `Bitset` `<<`/`>>`/`|`/`&`/`^` (`H4`, `H5`); `Perm.cycles`/`pow` (`H6`); `Char.is_lower`/`is_upper`/`is_digit`/`lower`/`upper`/`digit` (`G3`, `L9`).

---

### 2.4 Core Control Flow, Expression Uniformity, and I/O Completeness

- **Uniform expressions & filters:** `if`, `if let`, multiline `if`/`elif`/`else`, and `match` are all expressions (`G19`, `S7`, `P9`); one-line `if` without `else` works in statement-position `match` arms (`F24`); `for x in xs if cond:` filters loops just like comprehensions (`P9`, `9-T18`); `let` works anywhere in `and` chains for both `if` and `while` (`4-B12`, `S6`); `var (a, b) = (x, y)` binds mutable vars and `(a, b) = (b, a)` reassigns in parallel (`P2`).
- **Evaluation order & scope:** `for i in 0..xs.len` evaluates the range once (`H17`); `print` evaluates all arguments left-to-right before writing (`K2`); `compress(expr)` evaluates once (`S2`); nested comprehension `for`s scope left-to-right (`S9`, `9-T18`); top-level `return` exits the test/program (`S7`, `9-T13`, decision `6.3`); `unreachable` diverges (`G17`); last-use moves (`F23`); `f::<T>()` parses everywhere (`H2`).
- **Complete `read()` & formatting:** Tuple `read()` with forward-referencing bounds (`S8`, `N14`), mutable list `read(count: m)` (`N16`), record-field dependent bounds (`S4`, `K19`), `read()` into a place (`D11`), `read(none: 0)` / `read(none: '.')` and `?V` optional reads (`L20`, `D11`), packed 1-D/2-D `Char`-domain and single-char `@tags` reads (`G8`, `F3`, `4-B13`), a fieldless enum without `@tags` as a compile error that gives the `@tags(...)` line to add (`L26`, landed as 15.9), tagged payload enums (`S8`, `D11`, batch 2 gaps), `read_line()` / `read_chars(n)` (`D11`), header-prefixed `@multitest(count: tcs)` (`P14`), and string interpolation `"{e}"`, `"{x:.2}"`, `"{n:04}"` (`G4`, `10-T10`).
- **Range-checker precision:** `gcd(a, b) >= 1` when either arg excludes 0 (`G1`), constant shift folding `1 << 30` (`9-T18`), `assert` feeding flow facts (`G1`), `rand(lo, hi): lo..=hi` (`S5`), `sum`/`product` starting from `T.zero`/`T.one` (`H3`, `N6`), `Mod` + `Int` one-way reduction (`N5`), `if`-branch literal unification with `Mod`/`F64` (`4-B10`), and parenthesized range types `[D](lo..=hi)` parsed as range types rather than 1-field records (`N7`).

---

## 3. Bare-Core Fallback Ergonomics

Roughly 20–30% of real contest problems are weird ad-hoc, constructive, geometry, or simulation tasks (`cco16p2`, `dmopc21c9p4`, `cco04p2`, `cco04p4`, `cco17p3`, `noi03p2`, `ccc21s2`, `noip14p6`, `ccc18s3`, `noip09p4`) that do not fit any pre-packaged abstraction. Under contest pressure, a programmer must be able to ignore all high-level machinery and write plain imperative code — `Int`, `F64`, `List<T>`, `0..n` arrays, `while` loops, structs, recursion, `a.raw[i]`, `unchecked:` — and **never fight the language**.

### The Bare-Core Guarantee
> *Any algorithm writable in straightforward C++17 (plain arrays, integer indices, loops, stacks, queues, structs, recursion, floats) must be equally short or shorter in bare-core CP Lang, with zero forced domain declarations, zero type-gymnastics casts, and zero borrow-checker surprises.*

### Audit of Every Place the Bare Core Currently Fights You

Reading the ad-hoc, grid, and geometry ports across all 14 batches exposes eight concrete ways today's prototype punishes bare-core code — every one fixed by the unified core above:

1. **Destructuring `let` memory bug and `using namespace std;` collisions (`L0`, `P1`, `P2`, `P3`).** Writing an ordinary BFS worklist (`let (r, c) = todo[next]; todo.push(...)`) triggered a real ASan heap-use-after-free (`L0`); naming a variable `norm` broke `g++` (`P1`); `var (a, b) = (x, 0)` failed to make `a` or `b` mutable (`P2`); reading inside a top-level `while` loop swallowed the rest of `main()` (`P3`). Bare-core code must be 100% memory-safe and C++-collision-free by default.
2. **Plain `0..n` arrays and `List<T>` were second-class citizens.** `[0..=n]Int` failed to assign from a second `[0..=n]Int` (`4-B9`, `H21`); `List<T>` could not slice (`H20`, `S19`), splice (`D3`), `.dedup()` (`P14`), check `.is_empty` (`P14`), or binary-search (`D8`, `H19`, `S13`, `9-T18`) — forcing `cco04p2` to declare a throwaway `domain R = 0..rows.len` inside a loop!
3. **Broken basic loop & I/O semantics.** `for i in 0..out.len: out.push(...)` spun forever (`H17`); `print("!", ask(...))` printed `"!"` before asking (`K2`); `return` failed at top level (`S7`, `9-T13`); `let (n, m): (Int, Int) = read()` failed (`S8`); `read_line()` did not exist (`D11`); `"{a}-{b}"` printed literal braces (`G4`).
4. **Missing bare numeric, struct, and container primitives.** Ad-hoc geometry, DP, and implementation ports hit missing `F64` (`F15`, `K17`, `P4`), missing `U64`/`I128` (`G9`), missing `popcount`/`ctz` (`H12`, `P11`), missing `sign` (`G20`), missing `Char` predicates and ranges (`G3`, `L9`), missing in-file `struct` methods (`P8`), `HashMap<(Int, Int), Int>` failing `g++` (`K1`), and records with enum fields failing `g++` (`G25`).
5. **Inconsistent names and non-mutable `.last`.** `xs.len` vs `heap.len()`, `xs.last()` returning a copy so `cover[cover.len - 1].r max= s.r` repeated indexing (`S6`), `while let top = xs.last and top > x` rejected (`S6`), and `for ... if cond:` rejected (`P9`, `9-T18`).
6. **Borrow/disjoint-lend friction on imperative array mutations.** Swapping two computed indices (`swap(mut a[i], mut a[j])`, `H7`, `F27`) or merging two buckets (`2-B6`) should never force a proof mid-contest: `a.swap(i, j)` and `take(mut a[i])` are one-token escape hatches surfaced right in the error text.
7. **Forced boilerplate on simple ad-hoc data.** `ccc21s2` was longer than C++ because it added an `enum` where a `Bool` sufficed; `cco04p2` hand-parsed `3-5` because `read_line()` and `Str.split('-')` were missing. Bare-core CP Lang should make plain `Bool`, `List<Int>`, `Str.split`, and `Int.parse` effortless.
8. **Silent binary-search footgun (`9-T12`) and opaque `?T` messages (`K15`).** Separating `first(...)`/`find` (linear) from `bsearch_first` (monotone, debug-checked) and explaining why `max(expr for i in 0..n)` is `?Int` makes bare-core code predictable on the first compile.

Before (`cco04p2` bare-core loop today — throwaway domain, triple-indexed `.last`, four `write` calls):
```cp
for t in tiles.rev:
    domain R = 0..rows.len
    if let r = R.first_where(r => rows[r.idx][rows[r.idx].len - 1].b >= t.b):
        rows[r.idx].push(t)
    else:
        rows.push([t])
for row in rows:
    for t in row.rev:
        write(t.a); write("-"); write(t.b); write(" ")
    print()
```

After (clean bare-core CP Lang — zero high-level sugar, zero fights):
```cp
for t in tiles.rev:
    if let r = rows.bsearch_first(row => row.last!.b >= t.b): rows[r].push(t)
    else: rows.push([t])
for row in rows:
    print(" ".join("{t.a}-{t.b}" for t in row.rev))
```

### Every High-Level Abstraction Has a One-Line Bare-Core Escape Hatch
- **Domains (`domain V = 1..=n`)** -> `[0..n]T`, `List<T>`, or `a.raw[i]`.
- **Disjoint lends (`law disjoint`)** -> `a.swap(i, j)` or `take(mut a[i])`.
- **`Window` / `Slide` drivers** -> plain `while` loop + `Deque<T>`.
- **Typed `read()`** -> `read_line()`, `read_chars(n)`, `Str.split`, `Int.parse`.
- **Flat `@memo`** -> `@memo(hash)` or plain `HashMap<Key, Val>`.
- **Bounds/overflow checks in hot ad-hoc inner loops** -> `unchecked:` block (`7.1`).

---

## 4. `cpl stress` and Non-Typeable Problem Constraints

Typed `read()` declarations (`plan/18-key-features.md` §1) shine on standard constraints (`1..=n`, `Perm<V>`, `Tree<V>`). Yet real contests constantly impose **non-typeable constraints** that no static type system should try to encode:

- **Geometric constraints:** general position / no three collinear points (`cco16p2`), simple polygons, non-crossing line segments, non-overlapping circles (`jloi16p5`), convex hulls.
- **Graph-theoretic constraints:** planar graphs, connected graphs with extra edges (`9-T16`: 4 of 10 in Batch 9), symmetric adjacency lists (`btoi17p1`, `K4`), 2-edge-connected or chordal graphs.
- **Solvability & coupled constraints:** guaranteed-solvable instances, `ioi19p2` bushy trees where "no split exists" only appears on specific shapes (`9-T16`), every value in `1..=k` occurring (`nccc8s3`, `S4`), fixed-endpoint strictly increasing arrays (`joi16fp2`, `S4`), permutations with a fixed first element (`ccc21s4`, `F26`), or coupled multi-line inputs where `m` depends on the generated graph.

Several scouts proposed adding type wrappers or `read()` keyword flags (`read(increasing)`, `read(distinct)`, `read(covers: V)` in `S4`; `Symmetric<[P]List<P>>` and `Span<D>` in `K19`; `read(gen: tree)` / `connected` / `dag` in `9-T16`; a fixed-first hook on `Perm` in `F26`). **We must reject turning types and `read()` into a constraint-solver DSL.** Instead, `cpl stress` needs **three minimal, general tiers** with zero type bloat:

### Tier 1. Left-to-right dependent bounds + five universal `Read + Gen + Shrink` wrappers
1. **Dependent bounds everywhere (`K3`, `H1`, `S4`, `S8`):** Any scalar, tuple element, or record field range may reference earlier bindings or earlier fields of the same record (`let (n, k): (1..=2e5, 1..=n) = read()`, `[q](l: 1..=n, r: l..=n)`). Evaluating bounds left-to-right inside the default generator eliminates 70%+ of `where`-rejection waste (`cco17p5`, `cco22p1`, `ioi06p2`, `sac22cc5p4`) with zero new types.
2. **Only five universal stdlib `Read + Gen + Shrink` structs:** `Perm<V>`, `Range1<V>` (ordered pair `l <= r`), `Tree<V>`, `Distinct<[I]T>`, and `Sorted<[I]T>`, each defined in userland as a sealed struct carrying a `@gen` repair/builder and a `law` checked on `read()` (`R5`). Plus a tiny `std.gen` helper module of plain functions (`std.gen.tree_edges`, `std.gen.connected_edges`, `std.gen.dag_edges`, `std.gen.shuffle`) usable with `read(gen: ...)` (`9-T16`). Stop here — never add `PlanarGraph`, `NonCrossingSegments`, or `FixedFirstPerm` types.

### Tier 2. Constructive `gen:` + whole-input `@gen` + honest `where` diagnostics
1. **Unified codec for `read(gen: f)` (`K4` fix):** Any value returned by `f` is serialized by the exact same `Write`/`Read` codec as the declared type (fixing `K4` where `[P]List<P>` omitted list counts).
2. **One minimal hook for coupled / non-typeable constraints — whole-input `@gen`:** When multiple top-level `read()`s are coupled (a planar graph whose edge count `m` comes from a triangulation, non-crossing segments with queries over them, or a guaranteed-solvable instance built backwards from a hidden answer), a single `@gen fn make_case(size: Int)` writes a complete valid input stream directly using ordinary CP Lang:

```cp
// Non-crossing segments: constructive, 100% acceptance, zero bloated types
@gen
fn non_crossing(size: Int):
    let n = rand(1, size)
    print(n)
    var x = 0
    for _ in 0..n:
        let x1 = x + rand(1, 3)
        let x2 = x1 + rand(0, 3)
        print(x1, rand(-size, size), x2, rand(-size, size))
        x = x2

// Planar graph: random spanning tree + non-crossing chords on a cycle (outerplanar)
@gen
fn planar_graph(size: Int):
    let n = rand(3, size)
    var edges = [(i, i + 1) for i in 1..n] + [(n, 1)]
    // recursively add chords inside polygon faces -> guaranteed planar, 100% valid
    ...
    print(n, edges.len)
    for (u, v) in edges: print(u, v)

// Guaranteed-solvable instance: synthesize the answer first, then emit the problem
@gen
fn solvable(size: Int):
    let n = rand(2, size)
    let hidden = [1..=n] _ => rand(1, 10)
    print(n)
    for i in 1..n: print(hidden[i] + hidden[i + 1])
```

3. **Honest `where` diagnostics and branch coverage (`S4`, `9-T16`, `friction.md`):** Keep `where` for concise predicates with naturally high acceptance (like `cco16p2`'s `general_position` on a wide grid), with three mandatory tooling behaviors:
   - If `where` rejects >50% of candidates, `cpl stress` warns loudly naming the clause and rejection rate (`S4`) so a low-yield run never masquerades as a full pass.
   - `where` predicates are exempt from complexity/TLE warnings (`friction.md`).
   - `cpl stress` prints a one-line output/branch distribution summary (e.g. `188/200 printed -1`, `9-T16`) so degenerate generators like `utso15p5` and `ioi19p2` are caught immediately.

### What batch 16 adds: one promise statement, and a generator that is held to it

Batch 16 ported ten deliberately awkward problems (planar maps, non-crossing walls, a completable square, two interactive judges) and asked the second question of this section directly. Its answer is "not yet": `where` relates two values of one `read`, and none of the six promises it met has that shape. Its items `Q1` to `Q6` fit the three tiers and replace one piece of them.

**`assume e` is the general form of `where` (`Q1`).** A statement, legal after any `read`, over any `Bool` expression, with no sublanguage:

```cp
let walls: [W](u: P, v: P) = read()
assume all(pts[w.u].x == pts[w.v].x or pts[w.u].y == pts[w.v].y for w in walls)
assume no_two_cross(pts, walls)
```

On real input in a debug run it is an assertion, so a mistyped sample fails at the line that states the rule. In a judge build it is dropped. Under `cpl stress` with derived generators it is a filter, and a shrunk case is kept only if it still passes every `assume`. `read() where e` stays as the same thing attached to one `read`. This is one construct for a promise of any shape, and it keeps the promise a predicate: no `Planar<...>` type.

**With a `gen:`, the `assume` tests the generator (`Q2`).** Stress calls the generator and then evaluates every `assume`; a failure is reported as "the generator broke the promise", with the case, and is a bug in the generator and not in the solution. That is what makes a hand-written constructive generator safe to trust, and the `assume` may be as slow as a brute force because it never runs on the judge. A `gen:` function takes no arguments and reads the values already in scope, so a generator for the third line of input depends on the first two; the whole-input `@gen` above is then needed only when one function must emit several reads at once.

**Generate from a witness (`Q4`).** A `gen:` may return a `witness` beside the input: the answer it built the input from. Stress runs the co-located `checker` on the witness first, which tests generator and checker against each other, and from then on a solution that prints "no answer" on a case with a witness has failed. Without it a greedy that gives up too early passes. `gen.near(valid, moves: 1)` is the reverse: one random legal edit of a valid instance, for problems whose interesting inputs are almost valid.

**The usual shapes are plain functions in `std.gen`, each with its predicate (`Q3`, `9-T16`).** `balanced(len, kinds)`, `planar_faces(n)`, `disjoint_segments(m, box)`, `tree_edges`, `connected_edges`: twenty to forty lines of CP Lang each, none known to the compiler, and each enters the library when a second port wants it.

**An interactor's hidden state gets the same two hooks (`Q5`).** `fn gen(n) -> Judge` in place of the derived one, and `fn valid(self) -> Bool`, checked on every generated and every shrunk judge; call limits that depend on `n` live in the `protocol`, which is where §5 puts the budget of `K21`.

**A `stress` narrowing clause for coarse, tie-rich ranges (`Q6`).** Wide legal ranges like `0..=1000` with two decimals almost never produce equal distances or collinear points, so `<` vs `<=` bugs slip past stress. A `stress` clause on `read` narrows only what the derived generator draws without changing what the program accepts (`stress x in 0..=3 by 0.25, y in 0.25..=3 by 0.25`). The three read as one set: `where` and `assume` say what is legal, `gen:` says how to build it, and `stress` says where to look.

So the hooks are five, all general: dependent bounds, `assume`, `gen:` (with an optional witness), `stress` range narrowing, and choice-stream shrinking. The rejection-rate report of Tier 2 names the `assume` that rejects most and suggests a `gen:`.

### Tier 3. Automatic choice-stream shrinking for ANY constructive generator
If a contestant writes a 10-line planar-graph or guaranteed-solvable `@gen`, mutating the raw output text would break planarity or solvability. Instead, `cpl stress` uses **Hypothesis-style internal choice-stream shrinking**: every `rand(...)` draw inside `gen:` or `@gen` is recorded as a deterministic sequence of integers. On failure, `cpl stress` shrinks that **choice sequence** (lowering `size`, reducing draws toward 0, deleting choice blocks) and re-runs the user's own `@gen`!
- Every shrunk test case is **guaranteed valid by construction** (still planar, still non-crossing, still solvable) with **zero user-written `Shrink` code**.
- Any `where` predicate is re-checked on each shrink candidate automatically.
- Together with co-located `@brute`, `checker` (`9-T17`), and `interactor` (`L27`), this also protects against buggy accepted corpus submissions (`cco16p2`, `jloi16p5`, `cco17p3`, `ccc10s4` in Batch 12!).

---

## 5. Explicitly Rejected One-Off Special Cases

To enforce `decisions.md`'s "general before special" rule (G7, G8), the following 18 proposals from the 15 backlogs are **explicitly rejected**, with the general primitive that covers each one.

1. **Compiler-tracked reactive / auto-refreshing `SegTree` (`K13` second half).** Violates explicit costs (G9) and hides O(log n) updates inside plain `have[i] += by` writes. *Covered by:* `SegTree.of(op, leaf)` + explicit `runs.refresh(i, reach(i))` (`F7`), or a local `mut fn` / write-back projection.
2. **Compiler `where`-clause inversion for triangular indices (`F4`).** Synthesizing `r*(r+1)/2 + c` from `where c <= r` is an undecidable special case. *Covered by:* 10-line userland `struct Tri<D>` implementing `Index` + `Read`.
3. **Separate `ModR` operation-value API (`3.10`, `2-B9`).** Splits modular arithmetic and `Comb`/`prefix`/`Fenwick` into two idioms when `let P = m` already exists via `3.8`. *Covered by:* unified `Mod<M: Modulus>` supporting both `const P` and `let P = m`.
4. **Dedicated `pool Nodes: Node` keyword (`plan/13 M9`).** Adds a keyword for one container. *Covered by:* library `Pool<Node>` using the fresh-type binder (`nodes.Id`, same as `[domain]T`'s `s.Idx`) plus `nodes.freeze()`.
5. **Dedicated `domain Node = A(X) | B(Y)` sum syntax (`9-T8`).** Duplicates `enum` with a second sum-type declaration form. *Covered by:* ordinary payload `enum Node` deriving `Index` automatically (`2.16`, `K11`).
6. **Input-format DSL zoo in types and `read()` (`S8` literal separators `(a: Int, "-", b: Int)`, `S4` `read(increasing)`/`read(covers:)`, `K19` `Symmetric<...>`/`Span<D>`, `F26` fixed-first `Perm` hook).** Turns types into a regex/constraint-solver zoo that still misses planar/geometric constraints. *Covered by:* dependent bounds `(l: 1..=n, r: l..=n)`, the 5 universal `Read + Gen + Shrink` structs, `read_line()` + `Str.split`, and constructive `gen:`/`@gen` with choice-stream shrinking (§4).
7. **Naming binary search `first_where` / `last_where` (`F10`, `K6`, `9-T12`).** Proven live footgun: `ioi19p2` called `Spot.first_where(...)` expecting linear search and got a silent wrong answer. *Covered by:* `bsearch_first` / `bsearch_last` / `partition_point` (monotone, debug-checked) for binary search, and `first(...)` / `a.find(x)` for linear search.
8. **Auto-vivifying `m[k].push(x)` on a missing map key (`L17`).** Unlike `m[k] += 1` (monoid identity), auto-inserting on method calls makes read-only expressions like `m[k].len` silently mutate or behave inconsistently. *Covered by:* `(m[k] ??= []).push(x)` using our general `??=` operator (`D5`).
9. **Implicit `?Int < Int` ordering where `none` means `+infinity` (Batch 12 "Seen again").** Breaks maximization (where `none` is `-infinity`) and reintroduces silent sentinel ordering (`plan/11` explicitly forbids `<` on `?T`). *Covered by:* `min=`/`max=` skipping `none`, or explicit `dist[q] == none or d < dist[q]!`.
10. **Colon functional-update syntax on arrays `board.with(p: none, q: who)` (`L24`).** Adds special parser syntax for a rare small-board operation. *Covered by:* ordinary userland method `board.with(p, none).with(q, who)` on `Copy` arrays.
11. **`kth_shortest(..., sources_at: 1)` (`2-B10`).** `sources_at: 1` is bespoke to `ioi11p4`. *Covered by:* an 8-line Dijkstra loop using `MinHeap` + `Graph.arcs`.
12. **`merge(a, b, by:)` yielding `Left`/`Right` (`F14`).** Batch 1 itself preferred an `enum` + `sorted`, and two-list sweeps are cleaner with `Deque.pop_*_while` (`S12`).
13. **Bare two-variable `for i, x in xs` without parens (`H8`).** Ambiguous with tuple destructuring `for (a, b) in pairs`. *Covered by:* `for (i, x) in xs.items`.
14. **`a.next_layer(fill, body)` closure-based DP step (`4-B4`).** Requires multi-statement block lambdas and indents the entire transition body to save one line. *Covered by:* structural range equality (`4-B9`) + `var next = below.blank(0)` followed by plain loops and `below = next`.
15. **Uninitialized `var a: [D]T` with debug shadow-bit traps and judge zero-fill (`S10`).** Creates debug-vs-judge divergence and shadow-memory overhead. *Covered by:* niche-packed `[D]?T = none` (same 4 bytes for domains/ranges/handles!), `[D] i => expr`, or `group`.
16. **Prelude `Order` enum hardcoded to `@tags("-1", "0", "1")` + `split` (`G22`), and `ask(..., limit: 170)` kwarg (`K21`).** `Order` collides with `std.algebra.Order<T>` and hardcodes one wire format; query budgets belong to the whole protocol (often shared across multiple `ask` sites), not one call. *Covered by:* local `@tags` enum + `sign(x)` + `group`, and budget enforcement in the co-located `interactor` block (`L27`).
17. **Flow-sensitive return-type narrowing of `l + 1` from `?I` to `I` (`H14`), separate `Steps`/`RangeSet` engines (`S18`, `10-T12`), and prelude micro-helper bloat (`G24` `.only`, `G11` `equal_pairs`, `F28` `D.first(k)`, `S15` `first_index`/`last_index`, `S19` `largest`, `L17` `unordered`, `4-B` `l.between(r)`).** Keep `l + 1` uniformly `?I` and elide the runtime check on `(l + 1)!` when flow facts prove `l < r`; unify `Steps` and `RangeSet` over ordered `Map`; express 1-line helpers directly (`assert fits.len == 1`, `sum(c * (c - 1) / 2 for (_, c) in counts(it).items)`, `(min(a, b), max(a, b))`, `l.r..r.l`).
18. **Built-in rectangle halving `Range<(X, Y)>.halve()` and `across` (`R11`).** Specific to one 2-D divide-and-conquer interactive problem (`btoi18p3`). *Covered by:* a 10-line local helper over `(Range<X>, Range<Y>)` using `r.mid` (`R1`).

---

## 6. Draft Updates for `decisions.md`

Every entry below is drafted as **Leaning** (pending Ryan's confirmation) in prose with its justification, following the three rules at the top of `decisions.md`.

### Existing rows to settle or revise

- **1.5 Float semantics (Open -> Leaning).** Canonical float type `F64` (IEEE-754 double, 8 bytes). NaN is not a valid `F64` value (producing one traps in local debug builds), so canonical quiet NaN is the niche and `?F64` is 8 bytes; `F64` implements `Ord` and `Niche`. Mixing `Int` variables with `F64` requires explicit `F64(x)`, while integer literals unify with `F64`. `==` compiles with a lint suggesting `close(a, b, eps:)`. `print(x)` defaults to 12 significant digits; `print(x, digits: k)` and `"{x:.k}"` format fixed decimals. *Justification:* unblocks 8 scout ports and 53 corpus float problems with zero NaN-ordering bugs and zero extra `seen` arrays in `@memo`.
- **1.7 & 1.8 `U64` / `I128` / bit ops (Leaning -> Ready to settle).** Confirm wrapping `U64` (full 64-bit shifts/masks) and explicit `I128`; expose `.popcount`, `.ctz`, `.clz`, `.bit_len`, `bit_ceil` on `Int` and `U64` (`G9`, `H12`, `P11`).
- **2.7 & 5.6 Growable domains, pools and handles (Leaning -> Revised).** Drop the `pool Nodes: Node` keyword (`plan/13 M9`). A growable arena is library struct `Pool<Node>` whose fresh handle `nodes.Id` uses the `[domain]T` fresh-type binder: a 4-byte niche-packed monotone `Handle` with check-free indexing into `nodes` and `PoolMap<nodes.Id, V>`, plus `let (Node, frozen) = nodes.freeze()` graduating it into an ordinary `Index` domain (`D4`, `10-T10`, `P12`).
- **2.9 `v + k` / `v - k` on positions (Leaning -> Confirm, add `b - a` and `H14` rule).** Confirm `v + k` / `v - k` return `?V`, and `b - a` on two positions of one `Seq` returns `Int` (`P6`). The static type of `v + k` is always `?V` (never flow-narrowed); when flow facts prove `l < r`, `(l + 1)!` elides the runtime check (`H14`).
- **3.10 `ModInt` moduli (Settled 2026-11-19 -> Proposed revision).** Unify `Mod<P>` and `ModR` into one `Mod<M: Modulus>` struct using `3.8`'s `const` / frozen-`let` member (`type Z = Mod<1e9 + 7>` or `type Z = Mod<m>`). `impl Mul` dispatches via `if const` to `mul_mod` when `P > 3_037_000_499` (`10-T2`), `impl Read` streams arbitrary-length signed decimals (`G14`, `H9`), `Int` next to `Mod<M>` reduces one-way into `Mod<M>` (`N5`), and `sum`/`product` start from `T.zero`/`T.one` (`N6`). Retire `ModR` (`2-B9`, `P7`).
- **4.3 Sequence views (Settled -> Extended).** Standardize on `Window<I>` (FIFO drops) and `BagWindow<I>` (order-independent drops) traits in `std.window` powering fixed `slide`, variable `slide_while`/`slide_until`, 2-D `sliding`, and `mo`, with inline-callback overloads `add:`/`drop:` (under `5.4` sequential non-overlapping borrows). Rename binary search from `first_where` to `bsearch_first` / `bsearch_last` / `partition_point` (with debug monotonicity checks) and reserve `first(...)` / `a.find(x)` for linear search (`9-T12`). Two-pointer list draining uses `Deque<T>` + `pop_front_while`/`pop_back_while` (`S12`, `L4`).
- **5.4 Inline closures (Settled -> Clarified).** Two `inline` callback parameters in one call (such as `add:` and `drop:` in `slide`/`mo`) may mutate the same caller `var` when the callee invokes them sequentially and never nested inside each other.
- **5.9 `@memo` control (Open -> Leaning).** Layout follows `Index` automatically: flat `[Params]?R` when `Params: Index` and `at most 2^26 slots`, else `HashMap<Params, R>` (`4-B11`, `K5`, `L23`, `P4`). Override via `@memo(flat)` / `@memo(hash)`. `ask(...)` inside `@memo` is allowed for query-caching (`cco18p4` `K5`, `btoi18p3` `R10`), and `@memo` on a nested `fn` scopes its table to one call of the enclosing function (`tle17c5p6` `R10`); `read()`/`print`/`write` stay errors.
- **5.12 Projections: `lends` / `lend` (Settled -> Extended for `with` and write-back).** Add `with guard(): body` for `lends ()` scope guards (`L14`), which may skip lending when setup fails and always run post-`lend` teardown on `return`/`break`/`continue`/`?`. Clarify (and update `stdlib/README.md` lines 263–271) that post-`lend` write-back is the standard way `Counter`, `SegTree`, and `LazySegTree` support `t[i] += x` (`S14`).
- **6.8 Grader-style (IOI) problems (Open -> Leaning).** Support `@grader fn solve(...)` so `cpl build --grader` emits the judge entry function without `main`, while `cpl run`/`cpl stress` use a local driver (`F20`, `K20`, `9-T17`, 106 IOI problems).

### New rows to add

- **1.10 `??=` null-coalescing assignment (Leaning).** `place ??= expr` on `?T` evaluates `expr` only when `none`, writes it, and yields `T` (`D5`, `L17`).
- **2.16 Complete algebraic `Index` derivation (Leaning).** `Bool`, fieldless enums, bounded ranges `lo..hi` (with structural equality for identical written bounds in scope), `?T` (for `T: Index`), tuples/records of `Index`, payload enums of `Index`, and fixed-domain arrays `[D]E` (for `D, E: Index`, guarded by cardinality cap) all derive `Index` automatically (`L5`, `L23`, `4-B1`, `4-B9`, `4-B11`, `G18`, `K11`, `9-T8`, `P5`).
- **4.9 Uniform expression, filter, and evaluation-order rules (Leaning).** `if`, `if let`, multiline `if`/`elif`/`else`, and `match` are expressions; one-line `if` works in statement `match` arms; `for x in xs if cond:` filters loops; `let` works anywhere in `and` chains for `if` and `while`; destructuring `let` copies (never dangles) and `var (a, b) = ...` binds mutable vars; `for i in 0..xs.len` evaluates the range once; `print` evaluates all arguments before writing (`L0`, `G19`, `S6`, `4-B12`, `F24`, `H17`, `K2`, `P2`, `P9`, `9-T18`).
- **7.18 Non-typeable stress constraints (`@gen` + choice-stream shrinking) (Leaning).** Limit typed `Read + Gen + Shrink` to dependent bounds plus five universal structs (`Perm`, `Range1`, `Tree`, `Distinct`, `Sorted`). All other constraints (planar graphs, non-crossing segments, guaranteed-solvable instances) use constructive `gen:` or whole-input `@gen` shrunk automatically via choice-stream shrinking over `rand(...)` draws, plus a >50% `where`-rejection warning and output-branch coverage summary in `cpl stress` (`S4`, `K4`, `K19`, `F26`, `9-T16`).

- **1.6 Strings (Settled -> restated, with a test).** No change of design: `Str` is `[domain]Char`. Added: the package opens at every place a `Str` is named (record field, array element, loop variable), so indexing, slicing, iteration and the sequence views work there; `+`, `join` and `Str(chars)` build strings. *Justification:* the blind critic's thirteen failed probes and `H10`, `10-T3`; a string DP came out longer than its C++ for this reason alone.
- **2.13 Explicit type arguments (Settled 2026-11-19 -> to implement; 15.7 left it here).** `f::<T>(x)` in every expression position. `::` is not otherwise a token, so there is no ambiguity with `<`, and `f<P>(x)` keeps the error 15.7 gave it, with `f::<P>(x)` in its help line (`H2`, `H11`).
- **7.19 `assume` and `stress` narrowing (Leaning).** `assume e` after any `read`: an assertion in debug runs, dropped on the judge, a filter under derived generators, and the test of a `gen:` when there is one (`where` is `assume` attached to one `read`). A `gen:` may return a `witness`, and a `stress` clause narrows only the derived generator's domain to trigger ties and collisions without narrowing legal inputs (`Q1`, `Q2`, `Q4`, `Q6`).
- **7.20 Formats and continuation lines (Leaning).** `{e:fmt}` in a string literal, with an unsupported format a compile error; a line continues after a binary operator, after `=`, or inside an open bracket (`M1`, `M7`, `G4`).
- **7.21 The docs are checked (Leaning).** One cheat sheet, every snippet compiled in CI by the compiler it documents; a name the docs use and the compiler lacks fails the build (blind critic 5).


### Confirmation of existing leans (§§10–14)
Promote the proven leans from §§10–14 to **Settled**: bit compound assignments (`|=`, `&=`, `^=`), struct-owned domains (`Rooted<V>.Pre`), fresh domain returns (`scc`, `compress`, `D.chunks`), `if let` / `let ... else` chaining, open ranges `..i`/`i..`, `e?` in expressions, `min`/`max` skipping `none`, `.field` and bare operator call arguments, `x f= y`, missing map-key monoid identity (`F18`), read-only record width subtyping (`14.1`), and `use std.module.{Name}`.

---

## 7. Prioritized Implementation Spec for the Next Prototype Sprint

Execute in five dependency-ordered phases. Do not scatter new stand-ins across `proto/cpc` while ignoring the `cpc`/`cpi`/`crates/` split.

### Phase 1 (P0, Immediate): Close the 10 Bugs the Bug Sprint Left, and the Doc Drift
1. **Already landed in `proto/cpc`, sync `crates/cpl-check` (`decisions.md` §15):** `L0`, `K1`, `K2`, `K4`, `K5`, `H2` (as an error), `H17`, `L26`, `S1`, `S2`, `S3`, `S5`. Keep their tests, and port the 3 newly added negative checks (`enum_read_no_tags`, `generic_angle_call`, `minheap_by`) into `crates/cpl-check` so `cargo test` (`crates/cpl-cli/tests/corpus.rs`) is green again.
2. **Silent wrong answers still open:** rename the binary search from `first_where` to `bsearch_first` / `bsearch_last`, with the monotonicity check in debug runs (`9-T12`); fix `read()` inside a top-level `while` (`P3`); `Subset - Subset` is set difference (`4-B2`); `[Row, Col]Bit` reads and generates packed characters (`G8`); record-field dependent bounds are generated and checked (`S4`); in `proto/cpi`: `m[k] += x` on a missing key (`10-T11`), `min=`/`max=` on `?Int`, open slices, `Mod` with `Int` (`N5`), `sum`/`product` over `Mod` (`N6`), bounded values as numbers (`N7`).
3. **Invalid C++ and crashes still open:** no `using namespace std;` in emitted C++ (`P1`); `var (a, b) = ...` makes every name a `var`, and `(a, b) = (b, a)` assigns in parallel (`P2`, `N13`); `[D](lo..=hi)` is a range element type (`N7`); a struct gets `<` only when every field has one, so records with enums, `?T` or lists of them compile (`G25`, blind critic 3); a user `struct SegTree` or `struct Dsu` shadows the stand-in and never crashes the compiler (blind critic 3); `a.row(r) = row` copies (`4-B3`); `1 << 30` folds, `gcd >= 1`, `assert` gives a flow fact (`G1`, `9-T18`); `f::<T>()` parses (2.13); `x >>= k` and `x <<= k` beside `|=` (blind critic 7).
4. **Doc drift (blind critic 5):** one runnable cheat sheet under `site/`, every snippet compiled in CI; fix the six drifted forms it found (tuple `read()`, `first_true`, `if ... then`, enum payloads and `case`, `Deque`/`HashSet`, `mut fn`) by changing whichever side is wrong.
- **Verification gate:** One locked regression test per bug (including `scout_batch_12/bugs/let_then_push.cp` under ASan/UBSan!) in `crates/cpl-cli/tests/cases/` and `crates/cpl-cli/tests/programs/`.

### Phase 2 (P0, Architectural Unlock): End the `cpc` / `cpi` Split (`F1`, `F22`, `H3`, `P7`, `P8`)
Freeze new hand-written stand-ins in `proto/cpc/compiler.py`. Advance `crates/cpl-ir` + `crates/cpl-codegen` (whose Rust front end already type-checks all of `stdlib/` and `examples/local-lib/` per `decisions.md` §14!) to compile generic structs, in-file `struct` methods, traits, `lends`, `use std.*`, and `use local.*` directly to C++17 — OR add minimal generic-struct + `use` compilation to `proto/cpc` only if Rust codegen is more than one sprint away.
- **Verification gate:** Compile and differential-test `scout_batch_2/` (10 ports), `scout_batch_10/` (10 string ports), and `ioi16p6io`/`noi19p1` (`use local.ds.li_chao`) through ONE frontend + backend against their C++ originals at full judge sizes.

### Phase 3 (P1): Core Language Unification Sprint
Implement the five small kernel extensions in the Rust frontend/backend (and active prototype):
1. **Complete algebraic `Index` derivation:** `Bool`, fieldless enums, structural range equality (`[0..d]Z`), `?T`, tuples/records `(Row, Col)` everywhere (`for`, `[Cell]T`, `Dsu`, `bfs`), payload enums (`enum Node`), and finite arrays `[D]E` (`L5`, `L23`, `4-B9`, `4-B11`, `G18`, `K11`, `9-T8`, `P5`), plus flat-or-hashed `@memo` (`K5`).
2. **Unified `Mod<M>` via `const`/`let` `Modulus`** (`2-B9`, `10-T2`, `G14`, `H9`, `H11`, `P7`) and **`F64` + `U64` + `I128`** with quiet-NaN niche, literal unification, `print(..., digits:)`, and `"{x:.2}"`/`"{n:04}"` interpolation (`F15`, `4-B5`, `4-B10`, `G4`, `G9`, `K17`, `L25`, `P4`).
3. **`Pool<T>` + `p.freeze()`** using the fresh-type binder (`D4`, `10-T10`, `P12`), **`??=`** (`D5`), and **`with guard(): body`** over `lends ()` (`L14`).
4. **Control-flow & bare-core uniformity:** `match`/`if let`/multiline `if` expressions (`G19`, `S7`, `P9`), `for ... if cond:` (`P9`, `9-T18`), `while let ... and` + mid-`and` `let` (`S6`, `4-B12`), one-line `if` in match arms (`F24`), top-level `return` (`S7`, `9-T13`), `unreachable` (`G17`), nested comprehension `for`s (`S9`, `9-T18`), `l.r..r.l` boundary ranges and `b - a: Int` (`P6`), `List` slicing + `lends mut` `.first`/`.last` (`H20`, `S6`), and last-use move (`F23`).
5. **Input completeness:** tuple `read()` with forward-referencing bounds (`S8`, `N14`), mutable list `read(count: m)` (`N16`), record-field dependent bounds (`S4`, `K19`), tagged payload enum `read()` (`S8`, `D11`), `read(none: 0)` / `?V` reads and `read()` into a place (`L20`, `D11`), `read_line()` / `read_chars(n)` (`D11`), header-prefixed `@multitest(count: tcs)` (`P14`).
6. **`Str` opens wherever it is stored** (1.6 restated): indexing, slicing, iteration and views on a `Str` field or element, `[D](c: Char, x: Str) = read()`, `+` and `join`; `read(k)` / `read()` for `List<T>`; `{e:fmt}` holes; continuation lines (`H10`, `10-T3`, `M1`, `M2`, `M4`, `M7`, blind critic 2). Gate: the blind critic's `4331904` rewritten with `Str` is shorter than its library-stripped C++.
- **Verification gate:** `dpj`, `noi05p4`, `ccc20s5`, `cco19p1`, `ccc05s5`, `noip01p4`, `cco04p4`, `etsr23r3p2`, `noi06p4`, `noi15p3`, `cco17p3`, `dpv`, `noip14p6`, `wossoly21s3`, and `cco04p2` compile and pass differential stress tests cleanly.

### Phase 4 (P1): High-Leverage Userland `stdlib/` Sprint
Implement purely in `.cp` under `stdlib/` (each with `@test` brute-force suites and `cpl-prover` law verification):
1. **`stdlib/ordered.cp`:** Order-statistic tree backing `Map`, `Set`, `SortedBag`, plus unified `Steps` (`.runs` + `.values`) and `RangeSet` (`F13`, `2-B1`, `D1`, `S13`, `S14`, `S18`, `10-T12`).
2. **`stdlib/search.cp`:** Unambiguous `bsearch_first`, `bsearch_last`, `bsearch_first_some`, `bsearch_last_some`, `partition_point`, `r.mid`, `bisect` (on `F64`), `first_ge`, `last_le`, `argmin_convex`, `local_max`/`local_min`, plus linear `first`, `find`, `where`, `min_by`/`max_by`/`argmin`/`argmax` (`9-T12`, `R1`, `R2`, `R9`, `R10`, `K6`, `K7`, `D8`, `H19`, `L6`, `L13`, `P13`).
3. **`stdlib/window.cp` + `stdlib/deque.cp`:** `Window<I>` / `BagWindow<I>`, `MonoDeque`, `TwoStackFold`, `GroupWindow`, `slide`, `slide_while`, `slide_until`, 2-D `sliding`, `mo`, and `Deque<T>` with `pop_front_while`/`pop_back_while` (`K14`, `S12`, `2-B11`, `D7`, `L4`, `L8`, `L15`).
4. **`stdlib/grid.cp`:** `Dir4`, `Dir8`, `step`, `around`, `ray`, `lines`, `flipped`, `transposed` (`L1`–`L3`, `L22`), plus `D.chunks(k)` / `i.chunk(k)` (`D10`, `L12`).
5. **Stdlib completions:**
   - `stdlib/tree.cp`: DFS `Rooted<V, A>`, `pos -> Pre`, `at`, `up_arc`, `far`, `top_down`/`bottom_up`, `from_parents`, `tour`, `centroid`/`centroids`, `reroot`, `Hld` + `LazyHld` (`2-B2`, `2-B8`, `4-B14`, `H21`, `9-T1`–`9-T5`, `9-T10`, `9-T11`).
   - `stdlib/graph.cp`: `reach`, `bfs(limit:, source)`, automatic 0-1 BFS in `shortest`, `mst`, `two_colour`, `all_pairs`, `bridge_blocks`, `TwoSat<D>`, `Functional<V>` (`L7`, `L19`, `K8`, `G16`, `9-T6`, `9-T7`, `2-B12`).
   - `stdlib/algebra.cp`, `matrix.cp`, `linalg.cp`, `poly.cp`, `heap.cp`, `lazy.cp`, `segtree.cp`, `fenwick.cp`, `dsu.cp`, `subset.cp`: `Semiring` (`MaxPlus`/`MinPlus`), `Matrix<R, C, T>`, `Powers`, `row_reduce`/`det`/`rank`/`solve`/`XorBasis`, `Poly<T>`/`poly_at`/`convolve`, `Desc`/`desc`, `MinHeap`/`MaxHeap`/`Heap.by`/`peek`/`pop(k)`, `power`, `fold_others`, `RangeApply`, `?Int` lazy actions (`AddMin`, `AssignMax`, `AssignMin`, `Clamp`), `SegTree` write-back `at`, 2-D `Fenwick<(X, Y), T>`, `Dsu<V, T>`, `PotentialDsu<V, G>`, `sos_sub`/`sos_sup`/`mobius_*`, `Subset.shifted`/`add`/`remove`/`toggle`, `LiChao<F, V>` + `LiChaoArena<D, F, V>` (`N1`–`N4`, `N10`, `2-B3`–`2-B5`, `K9`, `S11`, `S14`, `D2`, `D6`, `D12`, `10-T7`, `P10`–`P12`, `L11`).
   - `stdlib/strings.cp`, `hashing.cp`, `bitset.cp`, `geometry.cp`, `math.cp`, `seq.cp`: generic `Indexed<I>` string algorithms, `Hash`/`StrHash`/`Hash2D`, `SuffixArray`, SA-IS, `overlaps`, `borders`, `Trie<A>` over `Pool`, `Bitset` `<<`/`>>`/`|`/`&`/`^`, generic `Pt<T>`, `Sieve.spf`/`multiplicative` (`mobius`/`phi`/`divisor_count`), `Congruence`, `Frac`, multi-var `quotient_blocks`, `Range.step(k)`, generic `compress` over `Ord`, `List.dedup`/`is_empty`/`splice`, `split`/`adjacent`/`cyc`/`zip`/`cycle`/`take`/`take_while`/`group`/`absorb`, `Perm.cycles`/`pow`, `Tri<D>`, `Char` predicates (`10-T1`–`10-T10`, `N8`, `N9`, `N11`, `N12`, `G3`, `G6`, `G7`, `G10`, `G12`, `G15`, `G21`, `H4`–`H6`, `K17`, `9-T9`, `9-T14`, `L9`, `L16`, `L18`, `P14`).
- **Verification gate:** Re-port representative problems from every batch (`noip18p3`, `noi04p1`, `etsr23r1p2`, `ioi01p1`, `ioi06p2`, `coci20c6p5`, `noi01p1`, `noi15p5`, `noi16p1`, `noi17p2`, `sac22cc3p5`, `ccc18s3`, `noi05p1`, `noip22p3`, `cmt21r2ztr`, `noi21p1`, `noip00p4`, `noi23p2`) on top of `stdlib/` and verify both raw and library-stripped LOC improve substantially while `python3 proto/cp.py test stdlib` and `cargo test` stay green.

### Phase 5 (P2): `cpl stress` Non-Typeable Constraints, Graders, Checkers, and Interactors
1. Implement whole-input `@gen`, choice-stream shrinking over `rand(...)` draws, the >50% `where`-rejection warning, complexity-lint exemption for `where`, and output/branch coverage reporting in `cpl stress` (§4, `S4`, `9-T16`).
2. Implement `assume` (filter, debug assertion, and test of a `gen:`), witnesses, `gen.near`, `stress` range narrowing, and `std.gen` (`Q1` to `Q4`, `Q6`); wire up co-located `checker`, `interactor` (hidden state with `gen` and `valid`, call limits in the `protocol`) and `@grader` blocks (`9-T17`, `L27`, `Q5`, `F20`, `K20`).
3. Update `stdlib/README.md` lines 263–271 to reflect write-back projections (`5.15`).
- **Verification gate:** Stress-test `cco16p2`, `btoi17p1`, `ccc21s4`, `ioi19p2`, `cmt21r2ztr`, `cco02p3`, `ioi10p6`, plus synthetic planar-graph and non-crossing-segment `@gen` tests, verifying automatic shrinking to minimal valid counterexamples.
