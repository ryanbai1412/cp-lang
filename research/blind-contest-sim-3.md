# Blind Full-Contest Simulation #3 (CP Lang 0.2.2)

Six fresh, previously unported AtCoder problems (ABC 476–481, 475–600 pts) solved **statement-first from scratch** in CP Lang 0.2.2 (`target/release/cpl` at commit `680633a`) without reading any C++ solution. Every solution was exercised through `cpl check`, `cpl run`, `cpl build`, `g++ -std=c++17 -O2 -fsanitize=address,undefined`, max-N timing/RSS benchmarks, 200-case clean stress runs, and a planted realistic bug run through `cpl stress --trace` with `@brute`/`@checker`.

## Contest Scoreboard

All 6 ports live in `proto/ports/sim3/` (with `proto/ports/sim3/cp.toml` pointing `lib = "../../../examples/local-lib"` for Problem 5).

| # | Problem | Topics Tested | Sol LOC | 1st-Draft Errors | C++17 Size | ASan/UBSan | Max-N Runtime / RSS | Stress (200 cases) & `--trace` |
|---|---|---|---|---:|---:|---|---|---|
| P1 | `p1_abc476_g.cp` (ABC476 G) | `@!multitest` + precomp + dyadic DP + `@brute` | 38 | 1 (`E0201` tuple bound) | 11.1 KB | Clean | `1.14 s` / 5.2 MB (`T=1e4, 1e18`) | 200/200 agree; bug caught (`l=3, r=7`), **8.6 MB trace truncated in precomp** |
| P2 | `p2_abc479_e.cp` (ABC479 E) | `Graph.undirected` + `Rooted` + subtree diff + `@brute` | 28 | 4 (`from_edges`, `E0501`, record fields) | 17.1 KB | Clean | `0.13 s` / 47.8 MB (`N=2e5`) | 200/200 agree; bug caught (case 1, `N=5`), 87 KB trace |
| P3 | `p3_abc481_f.cp` (ABC481 F) | Directed weighted graph + `Mod<998244353>` + layered BFS + `@brute` | 38 | **0** | 10.8 KB | Clean | `0.23 s` / 32.3 MB (`N=M=2e5`) | 200/200 agree; bug caught (case 24, `N=7`), 106 KB trace |
| P4 | `p4_abc480_f.cp` (ABC480 F) | Bitmask DP (`Subset<I>`) + submask `@brute` | 20 | 4 (`a.sum`, `s.lsb`, `~s`, brute scope) | 8.0 KB | Clean | **`0.18 s`** / 19.3 MB (`N=20`) | 200/200 agree; bug caught (case 1, `N=2`), 73 KB trace |
| P5 | `p5_abc481_g.cp` (ABC481 G) | **`examples/local-lib/math/poly.cp` NTT (`convolve`)** + D&C + `@brute` | 72 | 1 (**P0 `E0206`: `cp.toml` `[libs]` ignored**) | 21.8 KB | Clean | **`0.40 s`** / 28.7 MB (`N=120k`) | 200/200 agree; bug caught (case 1, `N=2`), 90 KB trace |
| P6 | `p6_abc478_e.cp` (ABC478 E) | `Graph.directed` + `std.graph.scc` + DAG DP + **`@checker` + `@brute`** | 30 | 1 (`let (domain C, comp)`) | 16.0 KB | Clean | `0.08 s` / 48.8 MB (`N=Q=2e5`) | 200/200 agree (built-in gen!); bug shrunk **32 -> 8 ints** (`N=3, Q=2`), 77 KB trace |

**Biggest wins since Simulation #2:**
- **NTT local library (`examples/local-lib/math/poly.cp`) is legitimately contest-ready in performance and bundle size.** Once resolved via `lib.toml`, `use lib.math.poly.{Z, convolve}` solved ABC481 G (`N = 120,000`, `O(N log^2 N)` divide-and-conquer) in **0.40 s** and **28.7 MB RSS**, and `cpl build` tree-shook the 403-line `poly.cp` down to only `ntt`, `convolve`, and `ntt_p` (**21.8 KB** total C++17, zero unused FPS functions emitted).
- **Synthesis #2 ergonomics landed cleanly across the core.** Mixed `Mod<P> * Int + Int` (`val * 10 + c`), `print(out, sep: "\n")`, `Char + Int` / `Char - Char`, `if best min= cand:` returning `Bool`, structural record subtyping on `Graph.directed` with extra weight fields, `@!multitest` top-level symmetry, and `std.graph.scc` returning a fresh topologically-ordered domain `C` all worked cleanly in both `cpl run` and `cpl build`.
- **In-file `@checker` + built-in shrinking is a killer feature when types match the domain.** On P6 (`p6_abc478_e.cp`), `cpl stress` needed zero generator code, caught the planted DP overwrite bug on case 142, shrank it from 32 integers down to `n = 3, q = 2, conds = [(0, 1, 3), (1, 2, 3)]`, printed the human-decoded input, and named the exact failing assertion (`checker: assertion failed: a[u] < a[v]`).

---

## Findings Ranked by Contest Time Lost

### 1. `[libs]` in `cp.toml` is not implemented in the Rust module loader (~8–10 min lost; P0 blocker for local libraries)

**Where:** P5 (`proto/ports/sim3/p5_abc481_g.cp` + `proto/ports/sim3/cp.toml`).

Both `examples/local-lib/README.md` and `site/content/stdlib.md` instruct contestants to register an external local library in `cp.toml`:

```toml
[libs]
lib = "../../../examples/local-lib"
```

```cp
use lib.math.poly.{Z, convolve}
```

Running `cpl check` / `cpl run` / `cpl build` fails immediately with:

```text
error[E0206]: no module `lib.math.poly`
  = help: `lib` is a library: a folder with a `lib.toml` that says `name = "lib"`, at or above this file
```

**Root cause:** `crates/cpl-check/src/modules.rs` (`library_root`) only walks ancestor directories looking for `lib.toml` and never reads `[libs]` from `cp.toml` (unlike `proto/cpi/interp.py`, which checks both). A contestant with a contest folder `abc481/` and a personal library `~/cp-lib/` cannot import their library at all without copying or symlinking `lib.toml` and the library directories into the contest folder. (We verified P5 end-to-end by symlinking `examples/local-lib/lib.toml` and `math/` into `/tmp/sim3_poly_test/`, leaving `proto/ports/sim3/p5_abc481_g.cp` + `cp.toml` as a ready regression test.)

**Fix:** Teach `FsLoader::library_root` in `crates/cpl-check/src/modules.rs` to walk ancestors for `cp.toml`, parse `[libs]`, and resolve relative paths against the directory holding `cp.toml` before falling back to ancestor `lib.toml`.

---

### 2. In-file `@gen` is silently ignored by `cpl stress`, and `--gen` disables shrinking (~6–8 min lost; affected P1, P2, P3, P4, P5)

**Where:** Every problem except P6 (5 of 6 problems).

Five of six contest problems have input invariants that plain independent type bounds cannot express:
- P1: `l <= r` with `l, r <= 1e18` (default generator picks huge `r - l`, timing out `@brute` with `TimedOut`).
- P2: `edges` must form a tree (default generator emits self-loops `1 1 / 1 1`, crashing `Rooted` with `assertion failed: order.len == V.K` and dumping a bogus `stress-1.in`).
- P3: every vertex reachable from `1`.
- P4: `sum(a) % m == 0`.
- P5: `a_i < b_i`.

When a contestant writes an in-file `@gen` function (as documented in `docs/one-page-cheat-sheet.md`):

```cp
@gen
fn mk(size: Int): print(7)
```

`cpl check` passes with zero diagnostics, and `cpl stress` **silently ignores `@gen`** and runs its built-in type-driven generator instead (verified via a repro where `n == 7` forces a WA, yet `cpl stress` reported `5 cases agree`). Even worse, dropping down to an external `--gen /tmp/gen.py` script works, **but completely disables counterexample shrinking** (P2, P3, P4, P5 all stopped on the raw unshrunk failing seed, whereas P6 without `--gen` shrank 32 integers to 8).

**Fix:** Wire in-file `@gen` into `cpl stress` so it runs on the same shrinkable choice stream (`rand(lo, hi)`) as built-in generation, keeping shrinking alive for trees, graphs, and constrained inputs. Until `@gen` is wired, emit a hard error or prominent warning whenever a file defines `@gen` and `cpl stress` ignores it.

---

### 3. `cpl stress --trace` truncates inside precomputation above `@!multitest` and skips `@checker`-only files (~4–5 min lost; P1 and P6)

**Where:** P1 (`@!multitest` with precomputation) and P6 (`@checker` without `@brute`).

Two separate `--trace` sharp edges showed up under real contest debugging:

1. **Precomputation exhausts the 40,000-step trace cap before `@!multitest` starts (P1):** `p1_abc476_g.cp` precomputes a `61 x 61 x 61` DP table `f` above `@!multitest`. On the shrunk failing case `1\n3 7`, `cpl stress --trace` wrote an **8.6 MB** `stress-1.trace.html` containing exactly 40,000 trace steps, all between lines 6 and 17 (the precomp loop). Line 18 (`@!multitest`) and the actual failing test case `l = 3, r = 7` were **never traced at all**.
2. **`--trace` emits no trace for `@checker`-only files (P6):** On constructive problems with multiple valid answers (like ABC478 E), the natural harness is `@checker` alone. Running `cpl stress --trace` printed:
   ```text
   warning: no trace: `--trace` runs the file against its own `@brute(main)` fn
   ```
   and also printed an empty `expected:` section in the terminal output. Only after adding a redundant `@brute(main)` alongside `@checker` did `--trace` produce a 77 KB trace.

**Fix:**
- In `@!multitest` traces, either skip/collapse step recording above `@!multitest` by default (snapshotting globals at the `@!multitest` boundary) or reserve a separate step budget for the failing test case.
- Support single-pane execution traces for `@checker`-only failures (showing `main`'s trace up to the failing output plus the failed `@checker` assertion), and omit the blank `expected:` block when no `@brute` exists.

---

### 4. `for i in mask:` over `Subset<I>` heap-allocates a `std::vector<int>` on every iteration (~3–4 min perf trap; violates zero-cost bar on P4)

**Where:** P4 (`proto/ports/sim3/p4_abc480_f.cp`).

Writing the idiomatic `Subset<I>` bitmask DP:

```cp
for s in Subset<I>:
    for i in Subset<I>.full - s:
        let ns = s.with(i)
        dp[ns] max= dp[s] + (if rem[ns] == 0: 1 else: 0)
```

compiled cleanly to 8.0 KB of C++17, but at `N = 20` took **0.18 s** (~6x slower than hand-written C++'s ~0.03 s). Inspecting the emitted C++17 revealed why:

```cpp
std::vector<int> t9 = [&]() -> std::vector<int> {
    int t5 = (int)((ll)(int)(Subset_I__len + -1) & ~(ll)s_2);
    std::vector<int> t6 = std::vector<int>();
    for (ll t7 = 0; t7 < I_len; t7++) {
        if (((ll)t5 >> (ll)(int)t7 & 1) == 1) t6.push_back((int)t7);
    }
    return t6;
}();
for (int i_3 : t9) { ... }
```

That allocates and frees **1,048,576 heap vectors** inside the inner loop (and at `N = 22` or `N = 23` would push a 2-second TL over the edge).

**Fix:** Lower `for i in mask:` directly to a zero-allocation bit loop:

```cpp
for (uint32_t m = mask; m; m &= m - 1) {
    int i = __builtin_ctz(m);
    ...
}
```

---

### 5. `Graph` / `Rooted` API friction, `E0501` nonzero array bounds, and cascading diagnostics (~4 min lost; P2 and P3)

**Where:** P2 (`p2_abc479_e.cp`) and P3 (`p3_abc481_f.cp`).

Four tightly coupled annoyances hit immediately when writing a tree solution from memory:

1. **No `Rooted.from_edges` constructor:** Writing `Rooted.from_edges(edges, root: V.first)` failed with `E0203`; contestants must construct `let g = Graph.undirected(edges)` and then `let t = Rooted(g, V.first)`.
2. **Non-zero structural array dimensions still fail (`E0501`):** Both `let edges: [1..n](u: V, v: V) = read()` (P2) and `var buckets: [1..=9]List<V> = []` (P3) fail with `E0501: cpl run and cpl build can't handle array dimensions that don't start at 0 yet`, forcing either an extra `domain E = 1..n` declaration or wasted `[0..10]` slots.
3. **Anonymous tuple edges fail inside `Graph.undirected` with a cascading line-1 diagnostic:** Passing `[E](V, V)` instead of `[E](u: V, v: V)` caused an unrelated error pointing inside `stdlib/graph.cp` / line 1 (`expected List<(to: Int, e: Int)>, found List<(to: Int, e: _)>`) rather than saying "edges must be records `(u: V, v: V)`".
4. **`let (domain C, comp) = scc(g)` parse error (P6):** Because `scc`'s signature is `-> (domain C, comp: [V]C)`, writing `let (domain C, comp) = scc(g)` is the natural guess and hits `E0106: expected a name or pattern after '(', found 'domain'` without suggesting `let (C, comp) = scc(g)`.

**Fix:** Support offset-based `[lo..hi]` / `[lo..=hi]` structural arrays in `cpl-ir`/`cpl-cpp` (eliminating `E0501`), add `Rooted.from_edges` or accept an edge array directly in `Rooted`, and add targeted diagnostic hints for `let (domain C, ...)` and unnamed edge tuples passed to `Graph.*`.

---

### 6. Dependent bounds rejected inside tuple `read()` and minor `Subset<I>` / `Array` naming guesses (~2–3 min lost; P1, P4, P5)

**Where:** P1 (`p1_abc476_g.cp`), P4 (`p4_abc480_f.cp`), P5 (`p5_abc481_g.cp`).

- **Dependent bounds fail inside tuple `read()`:** `let (l, r): (1..=1e18, l..=1e18) = read()` fails with `E0201: cpl run doesn't know the name l here`, forcing two lines (`let l: 1..=1e18 = read()` then `let r: l..=1e18 = read()`). The same limitation applies to record elements like `[I](a: 1..=M, b: a+1..=M)` in P5, which prevents the built-in stress generator from knowing `a < b`.
- **Small naming mismatches vs intuition (P4):**
  - `a.sum` failed (`E0203`); the working form is `sum(a[i] for i in I)`.
  - `~s` for subset complement failed (`E0302: expected Int, found Subset<I>`); the working form is `Subset<I>.full - s`.
  - `s.lsb` failed (`E0203`); the working property is `s.first` (which is clean once known, though a "did you mean `first`?" suggestion would save the lookup).
- **`rand` arity mismatch in temporary `.cp` generators (P1):** Writing `rand(1..=3)` failed with `E0303: rand takes 2 arguments, but 1 was given`; the working form is `rand(1, 3)`.
- **`cpl build --help` treats `--help` as a filename:** Running `cpl build --help` printed `cpl: can't read '--help'` instead of usage help (whereas `cpl stress --help` prints full help).

---

## Summary of Priority Actions

1. **P0:** Read `[libs]` from `cp.toml` in `crates/cpl-check/src/modules.rs` so external local libraries (`examples/local-lib/math/poly.cp`, personal CP libraries) work from any contest directory.
2. **P0:** Wire in-file `@gen` into `cpl stress` on the shrinkable choice stream (and stop silently ignoring `@gen`).
3. **P0:** Lower `for i in mask:` on `Subset<D>` to a zero-allocation `ctz` bit-loop instead of heap-allocating a `std::vector<int>` per mask.
4. **P1:** Fix `cpl stress --trace` for `@!multitest` precomputation (do not burn the 40k step cap before `@!multitest`) and emit a single-pane trace for `@checker`-only failures.
5. **P1:** Remove `E0501` on nonzero `[lo..hi]` arrays, allow dependent bounds inside tuple/record `read()` types, and add diagnostic hints for `let (domain C, ...)` and unnamed `Graph` edges.
