# Containers Matched Benchmarks

Matched-algorithm benchmarks comparing CP Lang containers against idiomatic C++17 (`g++ -std=c++17 -O2`, best of 3 runs, identical stdout verified).

Run all:

```bash
bash bench/containers/run.sh
```

Or a subset:

```bash
bash bench/containers/run.sh bitset interval_map dsu monodeque flatmap
```

## Results

| Benchmark | Workload | C++ reference | CP Lang | C++ | Ratio |
|---|---|---|---:|---:|---:|
| `hash_insert` | 500k random inserts/updates | `__gnu_pbds::gp_hash_table` (`splitmix64`) | 49.2 ms | 74.0 ms | **0.66x** |
| `hash_lookup` | 200k keys, 2M lookups (hit+miss) | `__gnu_pbds::gp_hash_table` (`splitmix64`) | 69.8 ms | 91.0 ms | **0.77x** |
| `hash_antihash` | 250k multiples of `126271` & `64` | `__gnu_pbds::gp_hash_table` (`splitmix64`) | 42.3 ms | 66.4 ms | **0.64x** |
| `bitset` | 200k-bit `subset_sums` + 200 `shift_or` + `&-chain` popcount | `std::bitset<200001>` | 7.5 ms | 8.2 ms | **0.91x** |
| `flatmap` | 300k `FlatMap.set` + 1M `FlatMap.get` | `__gnu_pbds::gp_hash_table` (`splitmix64`) | 62.6 ms | 59.6 ms | **1.05x** |
| `monodeque` | 2M sliding-window minimums (`w=1000`) | flat `vector<pair<int, ll>>` + head | 29.3 ms | 26.5 ms | **1.10x** |
| `interval_map` | 200k coalescing range assigns (`n=1e6`, `on_old`) | `std::map<int, int>` ODT | 41.6 ms | 74.5 ms | **0.56x** |
| `dsu` | 500k nodes, 500k `unite` + 1M `same`/`size` | union-by-size + path compression | 33.5 ms | 29.1 ms | **1.15x** |
| `sorted_range` | 100k inserts + 100k `.range(r)` `.first`/`.last`/`.len` + 20k range iters | matched randomized treap | 181.2 ms | 180.4 ms | **1.00x** |

Key optimizations landed while profiling this suite:
- `cpl::HashMap` / `cpl::HashSet` open addressing with 7-bit fingerprints, backward-shift erase, and full salted `splitmix64` (`hash_*`: `1.32x–1.76x` -> `0.64x–0.77x`).
- `MonoDeque` backed by flat `List<(Int, T)>` + `head` index (`monodeque`: `2.80x` -> `1.10x`); `Deque` power-of-two bitmask indexing (`& (len - 1)` instead of `% len`).
- `IntervalMap.assign` single-iterator `cpl::interval_assign` (`upper_bound(lo)` once + O(1) `erase(it)` + `emplace_hint` + in-place update) + `cpl::map_take` (`interval_map`: `2.49x` -> `1.14x` -> `0.56x`).
- `SortedBag` unchecked hot treap loops (`sorted_range`: `1.12x` -> `1.00x`).
- `FlatMap` unchecked masked probes (`flatmap`: `1.16x` -> `1.05x`).
