# Friction Backlog — Scout Batch 47: Hard CCC & CCO Part 2 (`ccc*`, `cco*`)

This report evaluates the locked CP Lang v2 specification (`docs/one-page-cheat-sheet.md` at `3723dca`) against ten previously unported accepted DMOJ CCC and CCO submissions (`proto/ports/scout_batch_47/`), spanning tree DP with walk reconstruction (`cco23p5`), parity-indexed lazy segment-tree DP (`cco24p5`), constructive tree rendezvous (`cco24p4`), patience-sort stack partitioning (`cco24p2`), tight-memory grader-interface graph reachability (`ccc20s2hard`), two interactive problems (`cco01p4io`, `cco07p5`), square grid rotation (`ccc18s2`), string hash-map constraints (`ccc22s2`), and ASCII character-table scoring (`cco04p1`).

Every problem has both an idiomatic locked-v2 port in `proto/ports/scout_batch_47/<prob>.cp` and a runnable prototype port in `proto/ports/scout_batch_47/today/<prob>.cp`. Running `python3 proto/ports/scout_batch_47/verify.py` compiles all 10 `today/` ports via `proto/cp.py build` (on temporary copies so tracked files remain unstamped) plus `g++ -std=c++17 -O2 -fsanitize=address,undefined -fno-sanitize-recover=all`, and verifies all 10/10 against the original accepted C++ submissions across 531 randomized differential, constructive, and live interactive-pipe runs (including full `100 x 100` rendezvous checks on `cco24p4`, stack push/pop simulation on `cco24p2`, tree-distance walk checks on `cco23p5`, and real bidirectional pipe interactors for `cco01p4io` and `cco07p5`).

## Dual LOC Measurements (Raw vs. Library-Stripped)

All counts below measure non-blank, non-comment code lines (excluding blank lines and `//` or `/* ... */` comments). For **library-stripped C++**, we remove only headers, `using namespace std`, debug macros, `init_io()`, `main()` wrappers, generic helpers (`reversed`, `sorted`, `rint`, `operator<<`, `read_int`), unused structs left in the submission (`BDS` brute-force oracle in `cco24p5`, unused `Vec` in `ccc20s2hard`), and standard reusable data structures (`Mint` and `FDS` lazy segment tree in `cco24p5`, `Tree` wrapper in `cco24p4`). All problem-specific logic remains intact. For the two interactive problems (`cco01p4io`, `cco07p5`), we report both solution-only v2 LOC and total v2 LOC including the co-located `@interactor` judge block.

- **`cco23p5`** (`5580402.cpp`): 419 raw C++ LOC, 392 library-stripped C++ LOC, **137 v2 LOC** (-67.3% raw / -65.1% stripped), 229 runnable `today/` LOC.
- **`cco24p5`** (`6435483.cpp`): 257 raw C++ LOC, 72 library-stripped C++ LOC, **51 v2 LOC** (-80.2% raw / -29.2% stripped), 92 runnable `today/` LOC.
- **`cco24p4`** (`6435484.cpp`): 174 raw C++ LOC, 79 library-stripped C++ LOC, **38 v2 LOC** (-78.2% raw / -51.9% stripped), 56 runnable `today/` LOC.
- **`cco24p2`** (`6435481.cpp`): 134 raw C++ LOC, 90 library-stripped C++ LOC, **48 v2 LOC** (-64.2% raw / -46.7% stripped), 74 runnable `today/` LOC.
- **`ccc20s2hard`** (`4324075.cpp`): 112 raw C++ LOC, 47 library-stripped C++ LOC, **22 v2 LOC** (-80.4% raw / -53.2% stripped), 51 runnable `today/` LOC.
- **`cco01p4io`** (`5034487.cpp`): 85 raw C++ LOC, 55 library-stripped C++ LOC, **28 v2 solution LOC** (50 with `@interactor`; -67.1% raw / -49.1% stripped on solution), 36 runnable `today/` LOC.
- **`cco07p5`** (`5120565.cpp`): 83 raw C++ LOC, 45 library-stripped C++ LOC, **31 v2 solution LOC** (47 with `@interactor`; -62.7% raw / -31.1% stripped on solution), 32 runnable `today/` LOC.
- **`ccc18s2`** (`4329456.cpp`): 86 raw C++ LOC, 51 library-stripped C++ LOC, **13 v2 LOC** (-84.9% raw / -74.5% stripped), 32 runnable `today/` LOC.
- **`ccc22s2`** (`4371343.cpp`): 62 raw C++ LOC, 35 library-stripped C++ LOC, **15 v2 LOC** (-75.8% raw / -57.1% stripped), 31 runnable `today/` LOC.
- **`cco04p1`** (`5430016.cpp`): 62 raw C++ LOC, 31 library-stripped C++ LOC, **17 v2 LOC** (-72.6% raw / -45.2% stripped), 29 runnable `today/` LOC.

**Batch totals across all 10 problems:** 1,474 raw C++ LOC (1,809 total lines) and 897 library-stripped C++ LOC drop to **400 solution-only v2 LOC** (**-72.9%** vs. raw C++, **-55.4%** vs. library-stripped C++), or **438 v2 LOC** including both co-located `@interactor` blocks (**-70.3%** vs. raw, **-51.2%** vs. library-stripped). Even the runnable `today/` prototype ports total **662 LOC** (**-55.1%** vs. raw, **-26.2%** vs. library-stripped).

The single biggest algorithmic win was `cco23p5`: even with zero external data structures to strip (392 stripped C++ LOC), `@memo`, `Rooted<V>`, `?(Int, V, V)` with `max=`, and higher-order `opt2`/`opt3` cut the solution by 255 lines (-65.1%). `I.bounds` in `cco24p5` (`i.l`, `i.r`, `I.first.l`, `I.last.r`, `u.r..`) also eliminated an entire class of off-by-one boundary arithmetic (`ind + 1` vs `n`) that dominated the C++ DP loop.

---

## Remaining v2 Friction and Multi-Feature Collisions

### 1. `@grader` + `Graph.directed(edges)` Memory Cliff on Tight-Memory Hard Problems (`ccc20s2hard`)

**Severity: P0 (causes MLE on judge constraints)**

In `ccc20s2hard`, `n, m <= 4000` produces up to 16,000,000 directed edges under a tight memory limit. The accepted C++ submission (`4324075.cpp`) had to hand-roll a flat `char buf[80000000]` byte arena and manually carve 32-bit `int*` slices by degree so the entire graph fit in ~80 MB.

In CP Lang v2, writing the natural idiomatic port hits two compounding memory collisions:

```cp
@grader
fn can_escape(n: 1..=4000, m: 1..=4000, grid: [0..=n, 0..=m]Int) -> Bool:
    let target = n * m
    domain Val = 1..=target
    var edges: List<(Val, Val)> = []
    for r in 1..=n:
        for c in 1..=m:
            let v = grid[r, c]
            if v in Val:
                edges.push((Val(r * c), Val(v)))
    let g = Graph.directed(edges)
```

First, materializing `edges: List<(Val, Val)>` before calling `Graph.directed(edges)` allocates 16M pairs of 64-bit integers (`256 MB`, peaking at `512 MB` during `push` reallocation) *on top of* the final CSR arrays (`head` + `to`). Second, DMOJ's C++ grader passes `std::vector<std::vector<int>>` (32-bit `int`); if `@grader` copies that into a 64-bit `[0..=n, 0..=m]Int`, the copy alone costs `128 MB`.

**Concrete fixes for v2:**
1. Allow domains whose statically proven upper bound fits in 32 bits (`domain Val = 1..=16_000_000`, or derived from `1..=4000 * 4000`) to narrow to `int32_t` storage inside arrays and `Graph` CSR tables, and let `@grader` bind `I32` parameters by zero-copy view over the judge's `std::vector<std::vector<int>>`.
2. Add a 2-pass streaming CSR builder to `Graph` (e.g., `Graph.directed_from(deg, fill)` or `Graph.directed_iter(n, gen)`) so huge sparse graphs build directly into 64 MB flat 32-bit CSR without materializing an intermediate `List<(Val, Val)>`.

---

### 2. `LazySegTree<B, Node>` Lacks Single-Leaf Field Modification & Leaves Inverted Ranges Unspecified (`cco24p5`)

**Severity: P0 (double `O(log n)` cost + silent-noop vs. panic ambiguity)**

In `cco24p5`, range updates add `+1` or `-1` to `node.mn`, while point updates overwrite *only* `node.ways = dp[i.r]` while preserving that leaf's accumulated `mn`. Because the locked v2 `LazySegTree` API only exposes `seg.set(idx, val)` (full leaf overwrite), the port must execute two traversals per step:

```cp
let cur = seg.fold(i.r..=i.r)
seg.set(i.r, Node(cur.mn, dp[i.r]))
```

That doubles the constant factor on `n = 500_000` and exposes a subtle semantic trap: if `LazySegTree` uses tag-permanent nodes (as `6435483.cpp`'s `FDS` does) instead of push-down, `cur.mn` includes ancestor tags, so passing `cur.mn` back into `set` would double-count ancestor tags unless `set` pushes down first. Providing `seg.update(i.r, node => Node(node.mn, dp[i.r]))` solves both in one traversal with unambiguous semantics.

Two more collisions surfaced in the exact same function:

```cp
var seg0 = LazySegTree<B, Node>(add_bad, [B] _ => Node(0, 0))
var seg1 = LazySegTree<B, Node>(add_bad, [B] _ => Node(0, 0))
...
seg.apply(d.r..=dup_end[cb, d]!, -1)
```

First, we wanted `var seg: [Bool]LazySegTree<B, Node>` so we could write `for cb in Bool: seg[cb]...` instead of duplicating `seg0`/`seg1` and passing `mut seg` to a helper. Initializing a `[D]T` array of non-`Copy` heap-owning data structures via `[Bool] _ => LazySegTree<B, Node>(...)` needs to be explicitly blessed in v2. Second, `6435483.cpp` explicitly guards `if (l > r) return;` in `update_a2(l, r, x)`. The v2 cheat sheet defines `lo..hi`, `lo..=hi`, `lo..`, `..hi`, and `..`, but never specifies whether an inverted range (`lo > hi`) passed to `seg.apply(lo..=hi, d)` or `seg.fold(lo..=hi)` is a valid empty range (no-op / monoid identity) or a runtime trap. Making inverted ranges empty no-ops eliminates defensive `if lo <= hi:` guards across every segment-tree problem.

---

### 3. No Stdlib Seeded RNG for Contestant Code (`cco07p5`)

**Severity: P1 (forces hand-rolled PRNG boilerplate in randomized algorithms)**

The v2 cheat sheet provides `gen` for property/stress test generation, but has no runtime seeded RNG for contestant code. In `cco07p5`, replacing C++'s one-line `mt19937 rng(1412)` + `rng() % bound` forced a 6-line manual `xorshift64` with explicit `U64` / `Int` conversions:

```cp
var seed: U64 = 1412
fn next_u64() -> U64:
    seed ^= seed << 13
    seed ^= seed >> 7
    seed ^= seed << 17
    seed

x = Int(next_u64() % U64(n - BLOCK + 1))
```

Randomized interactive probing (`cco07p5`), Treaps, randomized hashing, and Miller-Rabin are core CP staples. A tiny stdlib `Rng(seed)` with `rng.int(lo..hi)` (or `rng.pick(D)`) would collapse those 7 lines to 1 line with zero unsigned-cast noise.

---

### 4. `@interactor` I/O Primitives and Bare `Str` Indexing Under `ask()` (`cco01p4io`, `cco07p5`)

**Severity: P1 (underspecified spec surface + `Str` domain friction)**

Both `cco01p4io` and `cco07p5` benefit hugely from `ask(...)` on the contestant side (`let n: 1..=100 = ask("GetN")`, `let broke: 0..=1 = ask("Drop", l + i)`), which automatically formats, flushes, and parses typed responses. However, writing the co-located `@interactor` blocks revealed two spec gaps:

```cp
@interactor
fn judge():
    let hidden_n: 2..=1000 = read()       // reads test-case secret from judge stdin
    ...
    let cmd: Str = read_output()          // reads contestant stdout
    if cmd == "GetSize": print(hidden_n)  // writes to contestant stdin
```

First, `docs/one-page-cheat-sheet.md` lists `@interactor` without specifying how judge stdin (`read()`) is distinguished from contestant stdout (`read_output()` vs. `recv()`) and contestant stdin (`print()` vs. `send()`). Using `print()` inside `@interactor` to send to the contestant while `read_output()` reads from the contestant is easy to invert under contest pressure; explicit `send(...)` / `recv()` (or `sol.read()` / `sol.print()`) makes the two channels unmistakable.

Second, in `cco07p5`, the original C++ checked only the first character of `"True"`/`"False"` (`c == 'T'`). When we initially wrote `let tok: Str = ask("Look", l, r); tok[0] == 'T'`, it violated v2's strict domain rule because `tok` has type `Str` without a bound `domain` (`0` is an `Int`, not a domain index). Comparing `tok == "True"` fixed it cleanly, and `tok.first == 'T'` would also be domain-safe — worth making sure `Str.first` / `Str.last` are explicitly documented on `Str` alongside `[domain S]Char`.

---

### 5. Higher-Order Function Syntax, `@memo` + Reconstruction, and Custom Tie-Breaking on `max=` (`cco23p5`, `cco01p4io`)

**Severity: P1 (ergonomics + zero-overhead lowering guarantee)**

In `cco23p5`, the `k = 2` tree DP needs to pick 2 or 3 distinct children maximizing different DP transition callbacks:

```cp
fn opt2(ch: [V], f1: V -> Int, f2: V -> Int) -> ?(Int, V, V):
    let (t1, t2) = (top_k(ch, 2, f1), top_k(ch, 2, f2))
    var best: ?(Int, V, V) = none
    for (w1, v1) in t1:
        for (w2, v2) in t2 if v1 != v2:
            best max= (w1 + w2, v1, v2)
    best
```

Three v2 interactions collide here:

1. **User-defined closure parameters (`f1: V -> Int`):** The cheat sheet uses closures in stdlib calls (`sort(mut a, by: x => ...)`, `bsearch_first(lo..hi, x => ...)`) and `lends` callbacks, but does not lock the parameter type syntax (`V -> Int` vs `fn(V) -> Int`) or guarantee monomorphized template lowering (avoiding `std::function` heap allocation inside `O(n)` tree DPs).
2. **`max=` tie-breaking and non-`Ord` payloads:** `best max= (w1 + w2, v1, v2)` works cleanly on `?(Int, V, V)` because `Int` and `V` are `Ord`, and `none` acts as `-inf`. Yet in `cco01p4io`, we needed `<=` tie-breaking (preferring the largest step on ties to match `5034487.cpp`), forcing a fallback to manual `if best == none or cost <= best!.0: best = (cost, i)`. Similarly, if a reconstruction tuple holds a non-`Ord` enum or `Mod<P>`, `max=` fails to compile. A keyed update or `max_by` / `min_by` helper solves both cleanly.
3. **DP + reconstruction duplication:** In `cco01p4io`, returning a named tuple `(drops: Int, step: Int)` from `@memo fn dp(len)` completely eliminated the parallel `opt[]` table. In `cco23p5`, because `dp2` has 6 heterogeneous transition shapes (0, 1, 2, or 3 distinguished children), `build2` still had to duplicate all 6 branches (~58 LOC). Confirming `@memo` supports compact `enum` return types cleanly (without pointer-chasing or wasting cache space) lets complex tree DPs store a tiny tagged transition choice directly in the memo table.

---

### 6. Square-Grid Transforms vs. Separate `Row`/`Col` Domains & `.flipped` Axis Ambiguity (`ccc18s2`)

**Severity: P2 (surprising type rejection + rotation direction ambiguity)**

Writing `ccc18s2` with separate row and column domains immediately fails typechecking on rotation:

```cp
domain R = 0..n
domain C = 0..n
var g: [R, C]Int = read()
g = g.transposed.flipped   // type error: [C, R]Int cannot assign to [R, C]Int
```

Collapsing to a single square domain `domain N = 0..n` makes `g: [N, N]Int` compile in 13 lines, at the cost of giving up row/column mix-up checking inside `valid(a)`. That trade-off is reasonable for square matrices, **provided** grid transformations are unambiguously named: `g.flipped` on the cheat sheet does not indicate whether it flips horizontally (mirror columns) or vertically (mirror rows), which flips `g.transposed.flipped` between 90° CW and 90° CCW. Naming `g.flip_h`, `g.flip_v`, and `g.rotated_cw` eliminates a classic silent-WA pitfall.

---

### 7. ASCII `Char` Indexing Forces `ch - '\0'` Ceremony (`cco04p1`)

**Severity: P2 (noise on every byte-indexed frequency table)**

CP Lang locks `Char` to a single ASCII byte (`0..256`, zero UTF-8). Yet indexing a frequency or score table by character still forced `ch - '\0'` four times in 12 lines in `cco04p1.cp`:

```cp
var score: [0..256]Int = 0
var avail: [0..256]Int = 0
for _ in 0..m:
    let (ch, val, lim): (Char, Int, Int) = read()
    let code = ch - '\0'
    score[code] = val
    avail[code] = lim
```

And in `today/cco04p1.cp`, `proto/cpc` lacked `'\0'` and `.ord`, forcing `ch - 'a' + 97`. Allowing `[Char]Int` directly as a 256-entry fixed domain array (just as `[Bool]T` is a 2-entry array) lets contestants write `var score: [Char]Int = 0; score[ch] = val` with zero casts and statically guaranteed bounds safety.

---

### 8. Domain Index Parity (`.to_idx % 2`), Non-Mutating `a.rev` Equality, and Top-Level `return` (`cco24p5`, `cco24p4`, `cco24p2`)

**Severity: P2 (recurring small paper cuts across 4 problems)**

Three minor frictions recurred across multiple ports in this batch:

- **Parity of domain indices:** Both `cco24p5.cp` (`(i.to_idx % 2 == 1) == cb`) and `cco24p4.cp` (`(t.to_idx % 2 == 1) == ty`) had to drop out of domains via `.to_idx` just to branch on index parity. Having `Bool` work directly as a 2-element domain (`[Bool, V, T]V`, `for cb in Bool:`) plus clean index parity (`i.to_idx % 2` or `i.is_odd`) removes repeated boolean-conversion boilerplate.
- **Reversed copy vs. view comparison:** In `cco24p2.cp`, checking whether `a` equals reversed `b` required copying and mutating (`var ra = a.copy(); reverse(mut ra); if ra == b:`). Allowing value equality between a reversed view and an array (`if a.rev == b:`) or a pure `reversed(a)` avoids the temporary mutable copy.
- **Top-level `return` in prototype (`today/`):** Ideal v2 uses top-level `return` cleanly for early exits in `cco24p4.cp`, `cco24p2.cp`, and `ccc18s2.cp`. `proto/cpc` still rejects top-level `return`, forcing nested `if ... elif ...` pyramids in the runnable `today/` ports.

---

## Summary of Actionable Priorities for v2 / Rust Compiler

*(Cross-check against newly landed v2.1 `e7e6794`: `Bool` / `Char`-range indexing, `flipped_rows` / `flipped_cols`, bare top-level `return`, `min_by` / `max_by` tie rules, and `@grader` shared-domain parameters were just resolved in v2.1; the items below remain open post-v2.1.)*

1. **P0 — Tight-memory `Graph` + `@grader` (open post-v2.1):** 32-bit storage narrowing for bounded domains (`<= 2^31 - 1`), zero-copy `@grader` `I32` grid views, and a 2-pass streaming CSR constructor (`Graph.directed_from` / `Graph.directed_iter`).
2. **P0 — `LazySegTree` completeness (open post-v2.1):** Single-leaf callback update `seg.update(idx, f)`, explicit `[D]LazySegTree` array construction for non-`Copy` structures, and locked empty-noop semantics for inverted ranges (`lo > hi`).
3. **P1 — Stdlib `Rng` & explicit `@interactor` channels (open post-v2.1):** Minimal deterministic `Rng(seed)` (`rng.int(lo..hi)`), unambiguous `@interactor` I/O verbs (`send`/`recv` vs. judge `read()`), and `Str.first`/`Str.last`.
4. **P1 — Higher-order & `@memo` ergonomics (partially open post-v2.1):** Locked zero-overhead closure parameter syntax (`V -> Int` / `fn(V) -> Int`) and compact `enum` returns from `@memo` (tie-breaking on `min_by`/`max_by` landed in v2.1).
5. **P2 — Polish (mostly resolved in v2.1):** `a.rev == b` view equality and `g.rotated_cw` convenience remain open; `[Bool]`/`[Char]` indexing, `flipped_rows`/`flipped_cols`, and bare top-level `return` landed in v2.1.
