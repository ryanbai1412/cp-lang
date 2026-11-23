# Scout Batch 39 — Hard DMOJ Contest Classics (`aac`, `yac`, `wac`, `sac`, `dmpg`)

Ten unported hard problems drawn exclusively from the `aac*`, `yac*`, `wac*`, `sac*`, `bts*`, and `dmpg*` prefix bucket in `corpus/` (zero `bts*` submissions exist in the corpus; all ten have zero overlap with earlier `proto/ports/`, `scout_batch_*`, and `blind_critic_*` batches).

Each problem has two files:

- `<prob>.cp` — idiomatic CP Lang written against the locked **v2 spec** (`docs/one-page-cheat-sheet.md`, `3723dca`), rethinking each algorithm around v2 abstractions rather than translating C++ line by line.
- `today/<prob>.cp` — runnable version compiled by the current prototype (`proto/cp.py build`) and differentially verified against the original AC C++ submission on 150 small/medium random inputs under ASan + UBSan.

## Problems and Dual LOC

Non-blank, non-comment LOC comparing raw C++ (`C++ raw`), library-stripped C++ (`C++ stripped`, excluding debug/fast-IO/main boilerplate and reusable library structs such as `DSU`, `RMQ`, `Tree`, `LazySegTree`, `chash`, and rolling-hash helpers), idiomatic CP Lang v2 (`v2`), and runnable prototype CP Lang (`today`):

- `aac4p6` (`4227368.cpp`, deque sweep + circular shift): 181 raw / 48 stripped -> **27 v2** (-85.1% raw, -43.8% stripped), 46 today.
- `aac4p5` (`4140767.cpp`, constructive SCC graph via DSU + greedy heap merge): 169 raw / 114 stripped -> **49 v2** (-71.0% raw, -57.0% stripped), 84 today.
- `yac7p4` (`5705324.cpp`, dynamic active subtree with Euler preorder + binary lifting): 226 raw / 118 stripped -> **58 v2** (-74.3% raw, -50.8% stripped), 150 today.
- `yac6p6` (`5150025.cpp`, offline grid path DP with up to 5 cell skips via diagonal meet-in-the-middle): 170 raw / 137 stripped -> **72 v2** (-57.6% raw, -47.4% stripped), 126 today.
- `yac7p3` (`5705218.cpp`, functional graph cycle decomposition + max-depth rerouting): 122 raw / 95 stripped -> **58 v2** (-52.5% raw, -38.9% stripped), 83 today.
- `wac6p6` (`5122001.cpp`, multi-test interactive 3-color bin strategy): 113 raw / 79 stripped -> **43 v2** (-61.9% raw, -45.6% stripped), 70 today.
- `sac22cc4p4` (`4411200.cpp`, range-bounded subarray count rethought from 280-LOC `LazySegTree` to O(n) `MonoDeque` two-pointers): 280 raw / 34 stripped -> **19 v2** (-93.2% raw, -44.1% stripped), 33 today.
- `sac22cc5p5` (`4575631.cpp`, distinct `k x k` subgrids via 2D double rolling hash): 140 raw / 45 stripped -> **14 v2** (-90.0% raw, -68.9% stripped), 62 today.
- `dmpg16g2` (`4653887.cpp`, 5-letter rectangle inclusion-exclusion over DSU zero-runs): 166 raw / 113 stripped -> **58 v2** (-65.1% raw, -48.7% stripped), 98 today.
- `dmpg18s5` (`3476383.cpp`, online divisor-count range queries + point updates via sqrt decomposition): 132 raw / 116 stripped -> **36 v2** (-72.7% raw, -69.0% stripped), 79 today.

**Total across 10 problems:** 1,699 C++ raw / 899 C++ stripped -> **434 CP Lang v2** (**-74.5% vs raw, -51.7% vs library-stripped**) and **831 runnable `today/`** (-51.1% vs raw, -7.6% vs stripped).

## Verification

Run the differential harness across all 10 ports (150 small/medium random cases each, ASan + UBSan enabled on emitted C++17):

```bash
python3 proto/ports/scout_batch_39/check.py
```

Notes on verification:

- `aac4p5` is constructive; `check.py` checks feasibility parity (`-1` vs valid edge list) and independently runs Tarjan's SCC algorithm to verify exact edge count, simplicity, `x` SCCs, and all `q` required same-SCC pairs.
- `aac4p6`'s reference `4227368.cpp` allocates a 12 MB `DS` struct on the stack (`int data[3e6]`), overflowing the default 8 MB Linux stack; `check.py` raises `RLIMIT_STACK` for the reference binary only (the CP Lang port heap-allocates safely under the default stack).
- `wac6p6` is interactive on DMOJ; `check.py` verifies exact command-by-command transcript agreement with `5122001.cpp` across randomized 100-ball streams hitting all three strategy branches (234 / 459 / 78 cases per branch across 150 multi-test inputs), without running a live judge interactor process.
- Full-scale max-N timing/RSS benchmarks were not run in this scout batch; see `research/archive/frictions/friction-backlog-batch-39-dmoj-contests.md` for all v2 friction findings and prototype gaps.
