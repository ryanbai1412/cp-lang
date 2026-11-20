## Disagreement Triage & Resolution

### A. Complete Baseline (`321560a`) Disagreement Resolution (30 problems)

At baseline (`321560a`), 30 problems were skipped with `no input both programs agree on` (24 non-interactive buildable ports, 5 interactive problems, and 1 port using removed 0.2.2 `where` syntax). **All 24/24 non-interactive buildable baseline disagreements are now resolved and differentially verified at max-N.**

| Problem | Baseline Failure | Classification | Resolution / Evidence |
|---|---|---|---|
| `aac5p2` | `original exit -6` | Interactive protocol | Original uses online `?`/`!` interactor with `cin.exceptions(ios::failbit)`; cannot run on static stdin without an interactor harness. |
| `apio07p2` | `original exit -6` | Generator constraint bug | Added valid strictly-increasing coordinate max-N generator (`g_apio07p2`); now differentially verified and measured. |
| `btoi08p6` | `outputs differ` | Generator constraint bug | Added valid max-N grid generator (`g_btoi08p6`, distinct start/end cells); now differentially verified and measured. |
| `ccc02s4` | `original exit -6` | Generator constraint bug | Added valid bridge-crossing names/times generator (`g_ccc02s4`); now differentially verified and measured. |
| `cco07p2` | `original exit -6` | Generator constraint bug | Added valid 6-arm snowflake generator (`g_cco07p2`); now differentially verified and measured. |
| `cco09p4` | `original exit -6` | Interactive protocol | Original is interactive (`#ifdef LOCAL` offline wrapper absent under judge build); requires interactor harness. |
| `cco19p3` | `original exit -6` | Generator constraint bug | Added valid tree + weights generator (`g_cco19p3`); now differentially verified and measured. |
| `dmopc19c4p2` | `original exit -6` | Generator constraint bug | Added valid grid/start/target generator (`g_dmopc19c4p2`); now differentially verified and measured. |
| `dmopc21c7p3` | `original exit -11` | Generator constraint bug | Added valid DAG/cycle-free input generator (`g_dmopc21c7p3`); now differentially verified and measured. |
| `dmopc21c8p3` | `original exit -11` | `.cp` port defect + generator | Fixed O(N^2) string concatenation loop in `proto/ports/scout_batch_34/today/dmopc21c8p3.cp` (`print(perm, sep: " ")`) and added `g_dmopc21c8p3`; now differentially verified and measured. |
| `dmpg16s5` | `outputs differ` | Generator constraint bug | Added valid circular-array max-N generator (`g_dmpg16s5`); now differentially verified and measured. |
| `dpv` | `original exit -6` | Generator constraint bug | Added valid connected-tree generator (`g_dpv`); now differentially verified and measured. |
| `dpz` | `original exit -6` | Generator constraint bug | Added strictly-increasing heights generator (`g_dpz`); now differentially verified and measured. |
| `egoi22p1` | `original timeout` | Generator constraint bug | Added valid max-N generator (`g_egoi22p1`); now differentially verified and measured. |
| `egoi22p2` | `original timeout` | Generator constraint bug | Added valid tree/permutation generator (`g_egoi22p2`); now differentially verified and measured. |
| `egoi23p3` | `original exit -6` | Interactive protocol | Original runs interactive `query_filt` / `answer` loop with `assert(sz(cands) == 1)`. |
| `etsr23r2p3` | `original exit -6` | Generator constraint bug | Bounded inputs to 32-bit `int` matching original (`g_etsr23r2p3`); now differentially verified and measured. |
| `ioi00p1` | `original exit -6` | Generator constraint bug | Added valid length-N ASCII string generator (`g_ioi00p1`); now differentially verified and measured. |
| `ioi00p2` | `original exit -11` | Generator constraint bug | Added valid permutation/swap generator (`g_ioi00p2`); now differentially verified and measured. |
| `ioi05p4` | `original exit -6` | Generator constraint bug | Added valid permutation generator (`g_ioi05p4`); now differentially verified and measured. |
| `ioi07p1` | `original exit -6` | Interactive protocol | Original uses interactive `query(xl, yl)` / `answer(...)` loop. |
| `ioi08p4` | `original exit -6` | Generator constraint bug | Added valid interval/point max-N generator (`g_ioi08p4`); now differentially verified and measured. |
| `ioi11p3` | `original exit -6` | Generator constraint bug | Kept budget `b <= 1e9` to match original's 32-bit `int` wrapper (`g_ioi11p3`); now differentially verified and measured. |
| `nccc9s1` | `original exit -6` | Generator constraint bug | Added valid query counts generator (`g_nccc9s1`); now differentially verified and measured. |
| `noi07p5` | `original timeout` | Generator constraint bug | Added valid max-N necklace/query generator (`g_noi07p5`); now differentially verified and measured. |
| `sac22cc2p4` | `original exit -6` | Compiler bug + port workaround + generator | `@solution fn main():` emitted an empty `solve()` body (`bench/repros/bug_02_solution_fn_empty_solve.cp`). Moved solution body to top-level in `proto/ports/blind_critic_2/sac22cc2p4.cp` and added `g_sac22cc2p4`; now differentially verified and measured. |
| `sac22cc4jp4` | `original exit -6` | Generator constraint bug | Added valid permutation generator (`g_sac22cc4jp4`); now differentially verified and measured. |
| `yac6p3` | `original exit -6` | Stale 0.2.2 port syntax | Port uses removed `where` guard (`E0503`); requires port rewrite to 0.2.2 syntax. |
| `yac6p4` | `original exit -11` | Generator constraint bug | Added valid max-N input generator (`g_yac6p4`); now differentially verified and measured. |
| `yac7p5` | `original exit -6` | Interactive protocol | Original uses interactive tree query protocol with `cin.exceptions(ios::failbit)`. |

### B. Additional Disagreements & Compiler Bugs Surfaced Across Expanded Generators

Expanding `bench/corpus_generators.py` to 200+ new hand-written max-N generators uncovered concrete compiler bugs, O(N^2) codegen regressions, and port-level gaps:

| Problem | Symptom | Classification | Root Cause / Repro |
|---|---|---|---|
| `ccc03s4` | Port outputs empty stdout | **Compiler bug** | `@multitest fn solve():` emits empty `void solve() {}` -> `bench/repros/bug_01_multitest_empty_solve.cp`. |
| `btoi19p6` | Emitted C++ fails `g++` | **Compiler bug** | `List<Bool>.pop()` returns `std::optional<char>` rejected by `cpl::OptBool` -> `bench/repros/bug_03_optbool_pop_char.cp`. |
| `noi20p1` | Emitted C++ fails `g++` | **Compiler bug** | Unqualified `apply(best, mat)` collides with `std::apply` via C++ ADL -> `bench/repros/bug_04_adl_std_apply.cp`. |
| `btoi17p3` | Port timeouts (>20 s) at max-N | **Compiler perf bug (O(N^2))** | Bound method `tree.pos` copies entire `Rooted` tree into a `std::function` per plan -> `bench/repros/perf_01_by_value_vec_copy.cp`. |
| `cco16p1` | Port timeouts (>20 s) at max-N | **Compiler perf bug (O(N^2))** | `self.sz[x]` emits full `std::vector<ll> t0 = self.sz;` copy inside DSU loop -> `bench/repros/perf_01_by_value_vec_copy.cp`. |
| `cco22p3` | Port timeouts (>20 s) at max-N | **Compiler perf bug (O(N^2))** | Ternary `mine = loc == 0 ? a0 : a1` copies entire 100k-element `std::vector` per iteration -> `bench/repros/perf_01_by_value_vec_copy.cp`. |
| `noi15p1` | Port timeouts (>20 s) at max-N | **Compiler perf bug (O(N^2))** | `self.vals[x]` emits full `std::vector<ll>` copy per union-find operation -> `bench/repros/perf_01_by_value_vec_copy.cp`. |
| `dmpg16g2` | Port timeouts at max-N (matches small-N) | **Compiler perf bug** | Per-row `std::vector<std::vector<ll>>` event allocations inside O(32 N^2) grid DP (`perf_03` / `perf_01`). |
| `noip22p4` | Outputs wrap mod 2^64 as negative signed | **`.cp` port defect** | Port comment explicitly notes it uses signed `Int` instead of `U64` (exact 2^64 difference: `-4613361762843356744` vs `13833382310866194872`). |
| `dmopc21c5p4` | Outputs differ (`1 3 5...` vs `1 4 7...`) | **Multi-answer judge** | Both sequences are valid prime-step permutations; needs problem-specific validator instead of exact string diff. |
| `ioi05p2` | Outputs differ (`75468018` vs `1000000000`) | **`.cp` port defect** | Lazy segtree / coordinate compression query logic diverges from accepted C++ on negative-slope intervals. |
| `sac22cc3p5` | Outputs differ on distinct words | **`.cp` port defect** | Trie counting logic in `proto/ports/scout_batch_10/sac22cc3p5.cp` diverges from original C++ hash counter. |
| `noi02p4` | Port timeouts while original prints `-1` | **`.cp` port defect** | Port scans `m` up to `1e6` linearly without early impossibility check when `m` does not exist. |

---

## Ranked Compiler Performance-Tail Backlog (Ordered by Impact on p95, Max, Timeouts & RSS)

Inspected emitted C++17 against accepted original C++ across the top-20 runtime offenders, top-10 RSS offenders, and 4 O(N^2) timeouts. Every item below has a verified minimal repro in `bench/repros/`.

| Rank | Priority | Root Cause in Emitted C++17 | Affected Corpus Offenders (5-Run Measured Ratio / Impact) | Minimal Repro |
|---:|---|---|---|---|
| 1 | **P0** | **By-value `std::vector` / struct copies** on field reads (`std::vector<ll> t0 = self.sz`), container ternaries (`loc == 0 ? a0 : a1`), and bound-method captures (`Rooted t5 = tree; [self = t5]`) | **4 O(N^2) timeouts** (`cco16p1`, `noi15p1`, `btoi17p3`, `cco22p3`) + **#3 `ioi14p6io` (9.98x)**, **#5 `noi04p3` (5.98x)**, **#6 `dmopc21c4p2` (4.89x)** | `bench/repros/perf_01_by_value_vec_copy.cp` |
| 2 | **P0** | **`Map<K, V>` lowered to red-black tree `std::map`** with per-node heap allocation + double lookup (`cache.count(x)` then `cache.at(x)`) | **#1 `dmopc21c8p5` (36.80x, 155 -> 5695 ms)**, **#6 `dmopc21c4p2` (4.89x)** | `bench/repros/perf_02_map_rb_tree.cp` |
| 3 | **P0** | **Inner-loop heap `std::vector` allocation** for constant 4-neighbor direction literals (`[(0,1),(1,0),(0,-1),(-1,0)]`) and small non-escaping local lists | **#2 `shoi16p2` (21.33x, 27 -> 576 ms)**, **#8 `ioi09p6` (4.68x)**, **#10 `sac22cc2p4` (4.09x)**, **#18 `ucrpc21f` (2.80x)** | `bench/repros/perf_03_small_list_heap_alloc.cp` |
| 4 | **P0** | **Fixed small domain array `[0..K]T` inside a `struct`** lowered to heap `std::vector` (~80 B + malloc per node instead of inline `std::array<T, K>`) | **#4 `cco17p3` (7.99x runtime & #2 RSS 9.99x = 2323 MB vs 232 MB)** | `bench/repros/perf_04_struct_fixed_array.cp` |
| 5 | **P1** | **Type-erased `std::function` callbacks** inside `LazySegTree` / `SegTree` preventing inlining across millions of node merges | **#12 `ioi07p3` (3.49x)**, **#13 `cco21p5` (3.17x)** | `bench/repros/perf_05_std_function_segtree.cp` |
| 6 | **P1** | **Lifted `+` / `min=` on `?Int` disables `cpl::OptInt` niche packing**, forcing 16-byte `std::optional<ll>` across entire 2D/3D `cpl::Grid`s and structs | **#7 `yac6p6` (4.76x)**, **#10 `sac22cc2p4` (4.09x)**, **#16 `btoi17p2` (3.05x, RSS 6.08x)**, **#17 `cco23p2` (2.92x)** | `bench/repros/perf_06_optional_ll_grid.cp` |
| 7 | **P1** | **`@memo` emits `std::unordered_map`** even over bounded small integer ranges/domains instead of flat dense tables | **#11 `ioi09p4` (3.99x)**, **#19 `ccc18s4` (2.79x)**, **#20 `dpn` (2.64x)** | `bench/repros/perf_07_hash_memo.cp` |
| 8 | **P1** | **64-bit `std::vector<ll>` default width** for small-range ints and 3-state DP memos (8 bytes/cell vs 1-byte `int8_t` / 4-byte `int32_t`) | **#9 `cco05p6` (4.19x)**, **#14 `ccc20s4` (3.16x, RSS 3.50x)**, **#15 `coci21c1p2` (3.08x, RSS 6.85x = 145 vs 21 MB)**, **RSS #3 `cco22p4` (9.57x)**, **RSS #4 `ccc14s4` (8.93x)** | `bench/repros/perf_08_64bit_array_width.cp` |
| 9 | **P1** | **Full-stdin `cpl::read_all()` + full-stdout `cpl::out` buffering** + per-interpolation `cpl::Capture` strings | **RSS #1 `dmopc21c2p2` (37.42x = 131 vs 3.5 MB)**, **RSS #5 `ccc21s2` (7.25x)**, **RSS #8 `ioi09p3` (5.67x)**, **RSS #10 `noi19p5` (5.10x)** | `bench/repros/perf_09_read_all_rss.cp` |

### Estimated Impact on Corpus p95 & Max

Fixing the **4 P0 items** (`perf_01` through `perf_04`) eliminates all 4 O(N^2) timeouts, drops the worst runtime outlier from **36.80x to <5x**, drops worst RSS on `cco17p3` from **2.3 GB to ~250 MB**, and removes 7 of the current top-10 runtime offenders. Following up with **P1 items #5–#9** (`std::function` monomorphization, `cpl::OptInt` lifted ops, dense `@memo`, narrow integer storage, and 64 KB streaming I/O) directly addresses the remaining top-20 runtime tail and brings RSS p95 from **3.50x toward ~1.2x**.

### Untested / Out of Scope

- Interactive problems (`aac5p2`, `cco09p4`, `egoi23p3`, `ioi07p1`, `yac7p5`, plus grader-linked IOI submissions lacking `main()`) cannot be differentially timed on static stdin files and require an interactor/grader harness.
- Multi-answer constructive problems (`dmopc21c5p4`) require a custom validator rather than exact string diff.
- `crates/` and `stdlib/` were intentionally left untouched in this triage task; all compiler bugs and performance fixes are captured as minimal verified repros in `bench/repros/` for the compiler lanes.
