# CP Lang 0.2.2 Corpus Migration & Friction Backlog

Audit and migration of legacy pre-0.2.2 corpus ports in `proto/ports/` to clean, verified **CP Lang 0.2.2**. Only `proto/ports/` and `research/` were modified; `crates/`, `stdlib/`, `site/`, `docs/`, and `editors/` were left untouched so every remaining compiler or language rough edge is surfaced and logged here for root-cause fixes.

## Summary

Thirty-two corpus ports were migrated and verified end to end against the real Rust compiler (`cpl check`, `cpl build` compiled with `g++ -std=c++17 -O2 -fsanitize=address,undefined`, `cpl run`, and `cpl judge` for interactive ports) plus randomized differential tests against original accepted C++ submissions (or exact specification/DP oracles for constructive and round-trip problems). `cargo test -p cpl-cli --test corpus` passes cleanly (`2 passed; 0 failed`).

- **Interactive `@interactor` ports (7 of 7 using legacy `read_output()`)**: all 7 interactive ports in `proto/ports/` using `read_output()` were converted to locked 0.2.2 `@interactor fn judge(mut sol: Interactor)` with `sol.recv()`, `sol.send(...)`, and solution-side `ask(...)`. Non-comment code LOC moved from **319 to 366 (+47, +14.7%)**, because two ports (`egoi23p3.cp` and `dmopc21c2p3.cp`) were upgraded from non-runnable pre-0.2.2 sketches into fully executable Rust `cpl judge` programs with complete hidden-state validation.
- **Top 25 highest-friction non-interactive ports**: rewritten idiomatically using Tropical Optionals (`?Int` with `min=`/`max=`), `Marks<D>`, `Deque<T>`, `SortedBag<T>`, `compress(...)`, `Subset<D>`, `bsearch_first`, `Mod<P>`, `LazySegTree`, and top-level `@multitest` (on pure per-case ports without precomputation; none use `@multitest fn solve()`). Non-comment code LOC dropped from **1,293 to 1,026 (-267 LOC, -20.6%)**.
- **Combined total (32 ports)**: **1,612 -> 1,392 non-comment LOC (-220 LOC, -13.6%)**.

---

## Genuine 0.2.2 Compiler and Language Friction Backlog

These are real bugs and gaps observed directly against the Rust compiler (`crates/`) during migration, ordered by severity.

### P0 — `Subset<D>.len` silently lowers to domain size `2^|D|` instead of popcount (and `.is_empty` is missing)

In `proto/ports/scout_batch_42/today/mccc3j5.cp` and `proto/ports/scout_batch_43/today/etsr23r4p3.cp`, iterating `for mask in Subset<V>:` and checking `u in mask` works end to end in `cpl check`, `cpl build`, and `cpl run`, but `.len` on a `Subset<V>` value silently miscompiles:

```cp
for mask in Subset<V>:
    if all(not (u in mask and v in mask) for (u, v) in edges):
        ans max= mask.len       // BUG: evaluates to 2^n (16 for n = 4), not popcount!
```

`cpl check` and `cpl build` accept `mask.len` with no warning, and the generated binary prints `16` instead of `2` on a 4-vertex path graph because `.len` resolves to the domain cardinality (`1 << n`) rather than the subset's population count. Meanwhile `mask.is_empty` fails lowering with:

```text
error[E0203]: a `Subset<V>` has no member `is_empty` that `cpl run` knows
```

**Workaround used in migrated ports**: `popcount(mask.idx)` and `mask.idx == 0`.

**Root-cause fix for `crates/`**: In `cpl-ir` lowering, treat `.len` on a value of type `Subset<D>` as `__builtin_popcountll(mask)` (distinct from `Subset<D>.K`), and add `.is_empty` lowering to `mask == 0`.

### P0 — `LazySegTree(AddMin)` emits invalid C++ `void over` parameter in `cpl-codegen`

In `proto/ports/scout_batch_30/cco21p5.cp`, writing the documented 0.2.2 lazy segment tree:

```cp
use std.lazy.{LazySegTree, AddMin}
var tree = LazySegTree<Slot, Int>(AddMin)
```

passes both `cpl check` and `cpl build`, but the emitted C++17 fails to compile under `g++`:

```text
error: invalid use of type 'void' in parameter declaration
  Action_Int_Int_2 Action_Int_Int_2_new_Op_Min_16(
      ll id,
      const std::function<ll(ll, ll)>& compose,
      const std::function<ll(ll, ll, ll)>& apply,
      void over);
```

Zero-sized operator tags like `Op_Min` (`min`) lower to `void`, which is illegal as a C++ parameter type.

**Workaround used in `cco21p5.cp`**: construct an explicit `Action<?Int, Int>(0, (f, g) => f + g, (f, x, _) => x + f, over: Monoid<?Int>(none, opt_min))` using Tropical Optionals so no `INF` sentinel is needed and `g++` compiles cleanly under ASan/UBSan.

**Root-cause fix for `crates/`**: Omit zero-sized unit/operator-tag parameters from emitted C++ function signatures and call sites (or emit a `struct Unit {}` placeholder type instead of `void`).

### P1 — Uppercase single-letter `let L = ...` silently fails to enter value scope

In `proto/ports/scout_batch_32/today/996warmup8p3.cp`, declaring:

```cp
let L = 1_000_000_001
let hi_bound = 1_000_000_000 * 2 * L / n + 2
```

produces no error on line 1, then fails on every reference to `L` with a self-contradictory diagnostic:

```text
error[E0201]: no `L` in scope
   = help: declare it first: `let L = ...`
```

Because single-uppercase `let K = expr` is parsed as a size-parameter binding rather than a value `let`, any ordinary contest constant like `let L = ...`, `let N = ...`, or `let M = ...` disappears from value scope. Changing `let L = ...` to `const L = ...` fixes it immediately.

**Root-cause fix for `crates/`**: Either enter `let K = expr` into both size scope and value scope (so `L` is usable as an `Int` value as well as a dimension), or emit an immediate targeted diagnostic on `let L = <literal>` suggesting `const L = ...`.

### P1 — Domain type alias `type H = hs.D` works in array types but fails in `for cid in H:`

In `proto/ports/scout_batch_36/today/cco23p2.cp`, coordinate compression produces a fresh domain `hs.D`. Aliasing it with `type H = hs.D` works inside array declarations (`var by_h: [H]List<V> = []`), but fails when iterated directly:

```cp
let hs = compress(h)
type H = hs.D
var by_h: [H]List<V> = []   // OK
for cid in H:               // error[E0302]: `H` is a type, and a value is needed here
```

**Workaround used in `cco23p2.cp`**: iterate `for cid in hs.D:`.

**Root-cause fix for `crates/`**: Resolve domain type aliases (`type H = hs.D`) in `for x in H:` loop headers the same way named `domain` declarations are resolved.

### P1 — Top-level `@multitest` wraps the entire file instead of acting as a divider

In `crates/cpl-codegen/src/emit.rs`, file-level `@multitest` currently wraps all of `prog::solve()` in `for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();`. For pure per-test-case files (9 of our migrated ports), bare top-level `@multitest` works cleanly today and matches the upcoming divider syntax. However, in `proto/ports/scout_batch_48/today/egoi22p8.cp`, the 4x7 quadrant segment lookup table `seg` is built once before the test loop; placing `@multitest` after `seg` would still recompute `seg` on every test case today. Per Ryan's decision, `egoi22p8.cp` keeps `let tests: Int = read()` + `for _ in 0..tests:` until the top-level `@multitest` divider and top-level `return` land in `crates/`.

### P1 — Missing `Str * Int` repetition and seeded `Rng(seed)` in `cpl-ir` lowering

Two interactive ports hit stdlib/lowering gaps when upgraded from sketches to real Rust `cpl judge` executables:

- `proto/ports/scout_batch_19/egoi23p3.cp`: `Str * Int` (used to build repeated `"?"` padding strings in the original sketch) is not lowered by `cpl-ir`, requiring a 4-line `repeat_str` helper.
- `proto/ports/scout_batch_47/cco07p5.cp`: `Rng(1412)` is not available in Rust lowering (and 1-argument `rand(n)` fails); migrated to 2-argument `rand(lo, hi)`.

### P2 — No multi-statement closure blocks for `bsearch_first`

In `proto/ports/scout_batch_32/today/996warmup8p3.cp`, `bsearch_first(0..=hi_bound, mid => { simulate(mid); hl >= kl0 or hr >= kr0 })` fails to parse because CP Lang lambdas are single-expression only. Extracting a 3-line helper `fn hits_enough(mid: Int) -> Bool:` works cleanly, though allowing an indented or brace block after `=>` would eliminate the one-off helper in stateful binary-search problems.

### P2 — Remaining `read_output()` occurrences are `@checker` blocks (not `@interactor`)

A full grep of `proto/ports/` confirms **zero** remaining `@interactor` ports use `read_output()`. The 7 remaining `read_output()` files (`scout_batch_45/neerc10k.cp`, `scout_batch_45/iati21a23.cp`, `scout_batch_43/etsr23r4p1.cp`, `scout_batch_19/coci21c6p3.cp`, `scout_batch_19/ccc23s3.cp`, `scout_batch_19/dmopc21c5p4.cp`, `blind_critic_4/dmopc21c5p5_draft.cp`) are all constructive `@checker` blocks, awaiting first-class `@checker` execution in `crates/`. Two additional files (`scout_batch_16/ioi13p4.cp`, `scout_batch_16/ioi18p1.cp`) are IOI `@graded`/`protocol` design sketches that never used `read_output()`.

### P2 — Method-name ergonomics mismatches vs. older ports

Several stdlib methods in locked 0.2.2 differ from names used in older ports or drafts and caused initial check/build retries before checking `stdlib/`:

- `Marks<D>` uses `.add(x)` (not `.mark(x)`).
- `Deque<T>` uses `.push(x)` (not `.push_back(x)`).
- `SortedBag<T>` uses `.add(x)` (not `.insert(x)`).
- `Compressed<D, T>` uses `xs[i]` / `xs.get(i)` (not `xs.val(i)`).
- `bsearch_first` and `bsearch_last` require explicit `use std.seq.{bsearch_first}` rather than being in the prelude.

---

## Per-Port Migration Ledger (Exact Before -> After Non-Comment LOC)

### Interactive `@interactor` Ports (7 / 7 migrated, all verified with `cpl check`, `cpl build` + ASan/UBSan, and `cpl judge`)

- **`proto/ports/scout_batch_40/tle17c5p1.cp`** — `10 -> 10 LOC (+0)`: converted judge `read_output()`/`print` to `sol.recv()`/`sol.send(...)` with `@interactor fn judge(mut sol: Interactor)`.
- **`proto/ports/scout_batch_47/cco01p4io.cp`** — `50 -> 52 LOC (+2)`: migrated `@interactor` to `sol.recv()`/`sol.send(...)` + `ask(...)`; simplified optional DP transitions.
- **`proto/ports/scout_batch_47/cco07p5.cp`** — `47 -> 41 LOC (-6)`: migrated `@interactor` to `sol.recv()`/`sol.send(...)`, replaced `Rng(1412)` with `rand(lo, hi)`.
- **`proto/ports/scout_batch_44/dmopc21c2p3.cp`** — `23 -> 34 LOC (+11)`: upgraded from non-runnable sketch to verified `cpl judge` interactor using `sol.recv()`/`sol.send(...)` and `ask(...)`.
- **`proto/ports/scout_batch_34/dmopc20c6p6.cp`** — `80 -> 84 LOC (+4)`: converted legacy one-way interactive protocol to `ask(...)` + `sol.recv()`/`sol.send(...)` with full hidden jug simulation.
- **`proto/ports/scout_batch_19/ioi07p1.cp`** — `48 -> 51 LOC (+3)`: replaced unsupported pre-0.2.2 interactor sketch with verified `@interactor fn judge(mut sol: Interactor)` and `bsearch_last`.
- **`proto/ports/scout_batch_19/egoi23p3.cp`** — `61 -> 94 LOC (+33)`: upgraded from non-runnable sketch to verified `cpl judge` interactor with `sol.recv()`/`sol.send(...)` and `repeat_str`.

### Top 25 Highest-Friction Pre-0.2.2 Ports (all 25 verified with `cpl check`, `cpl build` + ASan/UBSan, `cpl run`, and 10–15 random differential cases each)

- **`proto/ports/scout_batch_39/today/yac6p6.cp`** — `128 -> 81 LOC (-47)`: replaced 3 `INF` DP tables (`dp`, `dp2`, `qans`) with Tropical Optionals `?Int` + `min=`, domain-indexed arrays, and compact `Query` tuple buckets. Verified 10/10 random diff cases vs `corpus/submissions/5150025.cpp`.
- **`proto/ports/scout_batch_36/today/cco23p2.cp`** — `102 -> 63 LOC (-39)`: replaced manual sort/dedup coordinate compression, manual binary search, and raw modular arithmetic with `compress`, `Mod<1_000_003>`, and `SortedBag<Int>.first_gt`. Verified 10/10 random diff cases vs `corpus/submissions/5579928.cpp`.
- **`proto/ports/blind_critic_4/ioi22p1.cp`** — `83 -> 53 LOC (-30)`: replaced manual binary search in `psum`, manual `list_max`, and `.from_idx(...)!` conversions with `bsearch_first`, `max(...)!`, `max=`, and `Col(i)`. Verified 10/10 random diff cases vs `corpus/submissions/4785724.cpp`.
- **`proto/ports/scout_batch_14/today/noi19p1.cp`** — `106 -> 80 LOC (-26)`: replaced `INF` DP sentinels and `-1` Li Chao tree node pointers with `?Int`, `min=`, and typed `Station` domain indexing. Verified 10/10 random diff cases vs `corpus/submissions/3790564.cpp`.
- **`proto/ports/scout_batch_30/noi06p3.cp`** — `80 -> 61 LOC (-19)`: eliminated `INF` sentinels in `pre`/`cur` DP arrays with `?Int` + `min=`, simplifying split-interval transitions. Verified 10/10 random diff cases vs `corpus/submissions/3868871.cpp`.
- **`proto/ports/scout_batch_23/today/ioi17p3.cp`** — `80 -> 64 LOC (-16)`: replaced 4 per-round `List.filled(n, false)` visited resets with O(1)-clear `Marks<V>` (`good`, `pushed`, `lose`) and 2 manual `head < q.len` queues with `Deque<V>`. Verified 10/10 random diff cases vs pre-migration reference.
- **`proto/ports/scout_batch_43/today/etsr23r4p3.cp`** — `44 -> 30 LOC (-14)`: replaced manual `1..(1 << n)` bitmask loops and bit tests with `for mask in Subset<V>:` + `for v in mask:` and `Rooted<V>.postorder`. Verified 10/10 random diff cases vs `corpus/submissions/5635626.cpp`.
- **`proto/ports/scout_batch_32/today/996warmup8p3.cp`** — `55 -> 43 LOC (-12)`: replaced manual `while lo < hi:` binary search and input loops with `bsearch_first`, `const L`, and domain-indexed `read()`. Verified 10/10 random diff cases vs `corpus/submissions/5324670.cpp`.
- **`proto/ports/scout_batch_30/noi06p1.cp`** — `56 -> 46 LOC (-10)`: replaced `INF`-initialized tree DP table with `[... ]?Int` and `min=`. Verified 10/10 random diff cases vs `corpus/submissions/5032743.cpp`.
- **`proto/ports/scout_batch_43/today/etsr23r3p3.cp`** — `52 -> 43 LOC (-9)`: replaced `INF` unreachable distances with `?Int` + `min=`, adopted top-level `@multitest`. Verified 10/10 random diff cases vs `corpus/submissions/5521298.cpp`.
- **`proto/ports/scout_batch_22/noi09p2.cp`** — `42 -> 34 LOC (-8)`: replaced manual binary search on overtake points with `bsearch_first` and adopted top-level `@multitest`. Verified 10/10 random cases against exact `O(n^2)` DP cost oracle and full line reconstruction validator.
- **`proto/ports/scout_batch_42/today/mccc3j5.cp`** — `18 -> 11 LOC (-7)`: replaced manual bitmask loop with `for mask in Subset<V>:` and `all(not (u in mask and v in mask) ...)`. Verified 10/10 random diff cases vs `corpus/submissions/4223449.cpp`.
- **`proto/ports/scout_batch_29/noi20p4.cp`** — `50 -> 44 LOC (-6)`: replaced manual min/max scan over active ingredients with `SortedBag`, kept `Bitset` subset-sum, adopted top-level `@multitest`. Verified 15/15 random constructive and impossible cases against exact subset-sum oracle and dish-weight validator.
- **`proto/ports/scout_batch_16/cco05p4.cp`** — `23 -> 18 LOC (-5)`: adopted top-level `@multitest` and streamlined prefix-sum scanning. Verified 10/10 random diff cases vs `corpus/submissions/4665767.cpp`.
- **`proto/ports/scout_batch_27/noi15p1.cp`** — `22 -> 18 LOC (-4)`: replaced manual `Map`-based ID assignment with `compress` + `Dsu<Var>` and top-level `@multitest`. Verified 10/10 random diff cases vs `corpus/submissions/3775616.cpp`.
- **`proto/ports/scout_batch_16/coci21c6p5.cp`** — `40 -> 37 LOC (-3)`: adopted top-level `@multitest` with clean `MinHeap` and optional pair state. Verified 10/10 random diff cases vs `corpus/submissions/4531869.cpp`.
- **`proto/ports/scout_batch_30/cco21p5.cp`** — `36 -> 33 LOC (-3)`: replaced negated `AddMax` + `INF` sentinel + `set_dp` workaround with Tropical Optional `LazySegTree<Slot, ?Int, Int>`. Verified 10/10 random diff cases vs `corpus/submissions/4554642.cpp`.
- **`proto/ports/scout_batch_48/today/egoi22p8.cp`** — `71 -> 68 LOC (-3)`: replaced manual binary search with `bsearch_first` and optional rotation minimum `best min= v`; kept `for _ in 0..tests:` after `seg` precomputation until the `@multitest` divider lands. Verified 15/15 random encode + all four 90-degree rotations + decode round-trips.
- **`proto/ports/scout_batch_44/today/dmopc21c9p2.cp`** — `17 -> 15 LOC (-2)`: replaced manual queue/head sliding-window list with `Deque<Char>` and top-level `@multitest`. Verified 10/10 random diff cases vs `corpus/submissions/4563973.cpp`.
- **`proto/ports/scout_batch_8/coci21c2p3.cp`** — `23 -> 21 LOC (-2)`: replaced manual `0..(1 << n)` bit-parity loops with `for flip in Subset<E>:` and optional `label: [V]?Int`. Verified 10/10 random diff cases vs `corpus/submissions/4226187.cpp`.
- **`proto/ports/scout_batch_29/shoi16p4.cp`** — `47 -> 46 LOC (-1)`: replaced manual bitmask inclusion-exclusion with `for skipped in Subset<Company>:` and `Mod<1_000_000_007>`. Verified 10/10 random diff cases vs `corpus/submissions/5635192.cpp`.
- **`proto/ports/scout_batch_30/cco17p2.cp`** — `29 -> 28 LOC (-1)`: replaced `INF`/`-INF` tuple sentinels with `(?Int, ?Int)` and `min=`/`max=`. Verified 10/10 random diff cases vs `corpus/submissions/6416978.cpp`.
- **`proto/ports/scout_batch_48/today/egoi22p1.cp`** — `13 -> 12 LOC (-1)`: adopted top-level `@multitest`. Verified 10/10 random diff cases vs `corpus/submissions/5198974.cpp`.
- **`proto/ports/scout_batch_10/noip20p2.cp`** — `28 -> 28 LOC (+0)`: replaced `[Letter]Bool` + `seen.fill(false)` reset with `Marks<Letter>` and adopted top-level `@multitest`. Verified 10/10 random diff cases vs `corpus/submissions/5445673.cpp`.
- **`proto/ports/scout_batch_15/noi09p4.cp`** — `48 -> 49 LOC (+1)`: replaced manual Kahn `head < q.len` queue and `[Cell]Bool` array with `Deque<Cell>` + `Marks<Cell>` + `FlowGraph<Node>`. Verified 10/10 random diff cases vs `corpus/submissions/5144326.cpp`.
