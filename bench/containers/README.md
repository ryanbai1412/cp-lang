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

Summary across all 13 benchmarks (`bash bench/containers/run.sh`): **p50 = 0.95x**, **max = 1.26x** (`sorted_set`).

| Benchmark | Workload | C++ reference | CP Lang | C++ | Ratio |
|---|---|---|---:|---:|---:|
| `hash_insert` | 500k random inserts/updates | `__gnu_pbds::gp_hash_table` (`splitmix64`) | 51.3 ms | 75.3 ms | **0.68x** |
| `hash_lookup` | 200k keys, 2M lookups (hit+miss) | `__gnu_pbds::gp_hash_table` (`splitmix64`) | 70.0 ms | 88.3 ms | **0.79x** |
| `hash_antihash` | 250k multiples of `126271` & `64` | `__gnu_pbds::gp_hash_table` (`splitmix64`) | 42.2 ms | 66.3 ms | **0.64x** |
| `bitset` | 200k-bit `subset_sums` + 200 `shift_or` + `&-chain` popcount | `std::bitset<200001>` | 8.1 ms | 8.4 ms | **0.96x** |
| `flatmap` | 300k `FlatMap.set` + 1M `FlatMap.get` | `__gnu_pbds::gp_hash_table` (`splitmix64`) | 58.6 ms | 61.4 ms | **0.95x** |
| `monodeque` | 2M sliding-window minimums (`w=1000`) | flat `vector<pair<int, ll>>` + head | 25.9 ms | 26.6 ms | **0.98x** |
| `interval_map` | 200k coalescing range assigns (`n=1e6`, `on_old`) | `std::map<int, int>` ODT | 41.8 ms | 74.9 ms | **0.56x** |
| `dsu` | 500k nodes, 500k `unite` + 1M `same`/`size` | union-by-size + path compression | 32.9 ms | 28.7 ms | **1.15x** |
| `sorted_range` | 100k inserts + 100k `.range(r)` `.first`/`.last`/`.len` + 20k range iters | matched randomized treap | 107.8 ms | 177.9 ms | **0.61x** |
| `sorted_set` | 100k inserts + 50k erases + 100k `rank` + 100k `nth` | `__gnu_pbds::tree` (`rb_tree_tag`) | 112.9 ms | 89.5 ms | **1.26x** |
| `ordered_map_set` | 100k inserts + 100k `first_ge` + 100k `range` ends + 20k range iters | `std::map` + `std::set` | 178.3 ms | 185.3 ms | **0.96x** |
| `min_heap` | 500k pushes + 500k push/pop + 500k drain | `std::priority_queue` | 115.8 ms | 125.7 ms | **0.92x** |
| `deque` | 200k prefill + 4M push/pop both ends + 2M random `get` | `std::deque` | 42.0 ms | 36.7 ms | **1.14x** |

Key optimizations landed while profiling this suite:
- `cpl::HashMap` / `cpl::HashSet` open addressing with 7-bit fingerprints, backward-shift erase, and full salted `splitmix64` (`hash_*`: `1.32x–1.76x` -> `0.64x–0.79x`).
- `MonoDeque` backed by flat `List<(Int, T)>` + `head` index (`monodeque`: `2.80x` -> `0.98x`).
- `Deque` raw `List<T>` ring buffer (no `OptInt` storage or per-`get` option unwrap, 2x smaller memory) + `Deque.reserve(cap)` preallocation + cached power-of-two `mask` + `unchecked:` hot ring ops (`deque`: `1.95x` -> `1.14x`; remaining ~5 ms vs `std::deque` is `assert 0 <= i < n` + `cpl::umod` on 2M random reads).
- `IntervalMap.assign` single-iterator `cpl::interval_assign` (`upper_bound(lo)` once + O(1) `erase(it)` + `emplace_hint` + in-place update) + `cpl::map_take` (`interval_map`: `2.49x` -> `1.14x` -> `0.56x`).
- `Dsu` single packed 32-bit negative-size array `List<-2_147_483_648..=2_147_483_647>` (4 B/vertex, 3x less memory) + `unchecked:` `find_ord` (`dsu`: `33.5 ms` -> `32.9 ms` wall, `30 ms` vs `27 ms` user; remaining ~3 ms is `cpl::umod` + `cpl::pos` in the benchmark's 3M random vertex casts).
- `SortedBag` / `SortedSet` single packed `BagNode<K>` array (32-bit `pri`/`left`/`right`/`par`, 40 B/node in one cache line instead of 7 separate vectors) + single-pass BST `insert_at`/`erase_at` (no sibling `pull` without rotation, no `contains` pre-check in `SortedSet.add`) (`sorted_set`: `2.55x` -> `1.26x` vs PBDS `rb_tree_tag`; `sorted_range`: `1.00x` -> `0.61x` vs C++ treap; remaining 1.26x vs PBDS is the expected randomized treap depth `1.386 log2 n` vs RB-tree `~1.05 log2 n`).
- `Iter::TreeRange` zero-allocation direct `std::set` / `std::map` iterator walk for `Set.range(r)` / `Map.range(r)` (`ordered_map_set`: `1.33x` -> `0.96x`, O(log n + k) instead of O(k log n) + `std::vector` copy).
- `FlatMap` unchecked masked probes (`flatmap`: `1.16x` -> `0.95x`).
