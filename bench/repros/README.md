# Compiler Correctness & Performance-Tail Minimal Repros (CP Lang 0.2.2)

Minimal standalone `.cp` reproducers surfaced during the Max-N Truth & Perf-Tail Triage across the 308-port corpus benchmark (`research/perf/corpus-p95-report.md`).

Verify all repros with:

```bash
for f in bench/repros/*.cp; do echo "=== $f ==="; target/release/cpl build "$f" -o /tmp/repro.cpp && g++ -std=c++17 -O2 /tmp/repro.cpp -o /tmp/repro && echo "COMPILED OK" || echo "FAILED"; done
```

## Compiler Correctness / Codegen Bugs

1. `bug_01_multitest_empty_solve.cp` — `@multitest fn solve():` emits an empty `void solve() {}` body and produces empty stdout (`ccc03s4.cp`).
2. `bug_02_solution_fn_empty_solve.cp` — `@solution fn main():` emits an empty `void solve() {}` body and produces empty stdout (`sac22cc2p4.cp`).
3. `bug_03_optbool_pop_char.cp` — `List<Bool>.pop()` returns `std::optional<char>` (from `std::vector<char>`), which fails `g++` when wrapped in `cpl::OptBool` (`btoi19p6.cp`).
4. `bug_04_adl_std_apply.cp` — unqualified calls to a user `fn apply(...)` taking a `std::vector` / array collide with `std::apply` via C++ ADL and fail `g++` (`noi20p1.cp`).

## Ranked Compiler Performance-Tail Repros (P0 -> P1)

1. `perf_01_by_value_vec_copy.cp` (**P0 — O(N^2) timeouts & 4x–10x slowdowns**) — by-value `std::vector`/struct copies on field projections, container ternaries, and bound-method captures (`cco16p1`, `noi15p1`, `btoi17p3`, `cco22p3`, `ioi14p6io`, `noi04p3`, `dmopc21c4p2`).
2. `perf_02_map_rb_tree.cp` (**P0 — #1 runtime outlier 34.82x**) — `Map<Int, Int>` emits red-black tree `std::map<ll, ll>` with double lookup (`count` + `at`) (`dmopc21c8p5`, `dmopc21c4p2`, `ioi04p4`).
3. `perf_03_small_list_heap_alloc.cp` (**P0 — #2 12.17x, #7 4.19x, #9 4.11x, #16 2.81x**) — small constant/local lists inside hot loops emit millions of heap `std::vector` allocations (`shoi16p2`, `ioi09p6`, `sac22cc2p4`, `ucrpc21f`).
4. `perf_04_struct_fixed_array.cp` (**P0 — #4 8.94x & #2 RSS 9.99x = 2.32 GB**) — fixed small domain array `[0..2]?Int` inside a `struct` emits heap `std::vector<std::optional<ll>>` per node (`cco17p3`).
5. `perf_05_std_function_segtree.cp` (**P1 — #12 3.55x, #14 3.10x**) — `LazySegTree`/`SegTree` emits type-erased `std::function` indirect calls on every tree node (`ioi07p3`, `cco21p5`).
6. `perf_06_optional_ll_grid.cp` (**P1 — #6 4.75x, #9 4.11x, #17 2.65x, #18 2.39x**) — lifted `+` / `min=` on `?Int` disables `cpl::OptInt` niche packing and forces 16-byte `std::optional<ll>` across the entire `Grid` (`yac6p6`, `sac22cc2p4`, `cco23p2`, `coci21c1p3`, `btoi17p2`).
7. `perf_07_hash_memo.cp` (**P1 — #11 3.58x, #15 3.00x, #19 2.38x**) — `@memo` lowers to `std::unordered_map` even over bounded small integer domains (`ioi09p4`, `ccc18s4`, `ioi04p4`).
8. `perf_08_64bit_array_width.cp` (**P1 — #13 3.28x & RSS #3–#10 5x–9x**) — 64-bit `std::vector<ll>` storage for small-range ints and 3-state memos causes 4x–8x RSS bloat and L3 cache thrashing (`coci21c1p2`, `ccc14s4`, `noip21p1`, `cco22p4`).
9. `perf_09_read_all_rss.cp` (**P1 — RSS #1 37.42x = 131 MB vs 3.5 MB**) — `cpl::read_all()` slurps entire stdin into RAM and `cpl::out` buffers entire stdout until exit (`dmopc21c2p2`, `ccc21s2`, `ioi09p3`, `noi19p5`, `apio09p1`).
