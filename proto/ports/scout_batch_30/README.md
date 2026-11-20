# Scout Batch 30 — Hard Multi-Topic Contest Classics

Ten unported multi-paradigm IOI, CCO, COCI, and NOI contest classics ported idiomatically to CP Lang against the unmodified `proto/cpc` prototype (zero changes to `crates/` or `proto/cpc/`, zero overlap with `scout_batch_1`–`scout_batch_31` or `blind_critic_*`), differential-tested on 100 random cases each against their accepted C++ submissions, and verified clean under `-fsanitize=address,undefined`.

## Problems

- `ioi99p2.cp` (`corpus/submissions/4896839.cpp`) — IOI 1999 P2 Hidden Codes. Byte-indexed `[domain S]Char` strings + reverse subsequence automaton + linear DP + constructive interval reconstruction. 45 CP LOC vs 83 raw / 52 library-stripped C++ LOC.
- `cco16p6.cp` (`corpus/submissions/4849378.cpp`) — CCO 2016 P6 Pirates. Game theory backward induction + shifted-origin `(count, weighted_sum)` `SegTree` + `first_where` binary search. 38 CP LOC vs 141 raw / 56 library-stripped C++ LOC.
- `cco17p2.cp` (`corpus/submissions/6416978.cpp`) — CCO 2017 P2 Cartesian Conquest. 2:1 rectangle tiling geometry + Euclidean step-jumping number theory + `HashMap<(Int, Int), (Int, Int)>` memoized min/max DP. 29 CP LOC vs 145 raw / 95 library-stripped C++ LOC.
- `cco21p5.cp` (`corpus/submissions/4554642.cpp`) — CCO 2021 P5. Graph DP + suffix minima + negated `AddMax` lazy segment tree. 36 CP LOC vs 136 raw / 47 library-stripped C++ LOC.
- `cco22p3.cp` (`corpus/submissions/4801968.cpp`) — CCO 2022 P3. Dijkstra over DP states + binary search on two disjoint interval schedules (eliminates a confirmed ASan heap-buffer-overflow in the original C++). 61 CP LOC vs 98 raw / 65 library-stripped C++ LOC.
- `coci21c1p5.cp` (`corpus/submissions/4086660.cpp`) — COCI 2021/22 #1 P5 Volontiranje. Patience-sorting LIS decomposition + flat packed min-`SegTree` + constructive backtracking DFS. 66 CP LOC vs 189 raw / 100 library-stripped C++ LOC.
- `noi06p1.cp` (`corpus/submissions/5032743.cpp`) — NOI 2006 Network Charge. Complete binary tree DP + ancestor majority bitmask + prefix-sum edge weights. 56 CP LOC vs 102 raw / 65 library-stripped C++ LOC.
- `noi06p3.cp` (`corpus/submissions/3868871.cpp`) — NOI 2006 P3. Independent upper/lower envelope decomposition + deduplicated local height candidates + `O(n * 25)` rolling prefix/suffix-min DP (rethought from global `10^5 x 25 x 2` memoized recursion). 80 CP LOC vs 110 raw / 73 library-stripped C++ LOC.
- `noi07p6.cp` (`corpus/submissions/3756979.cpp`) — NOI 2007 P6. `Rooted` tree (`depth`, `parent`) + deepest-first negated `MinHeap` greedy + constructive move sequence. 33 CP LOC vs 98 raw / 61 library-stripped C++ LOC.
- `noi19p4.cp` (`corpus/submissions/3800719.cpp`) — NOI 2019 Jump. Dijkstra shortest path + static 2D X-segment tree of y-sorted buckets with path-compressed next-alive links (rethought without `std::set`). 108 CP LOC vs 136 raw / 98 library-stripped C++ LOC.

Total code lines (excluding blank lines and comments): **552 CP LOC** vs **1,238 raw C++ LOC (-55.4%)** and **712 library-stripped C++ LOC (-22.5%)**.

## Verification

Run the differential test harness (80 small + 20 medium random cases per problem, with constructive validators for `ioi99p2` and `coci21c1p5`; builds temporary copies so tracked `.cp` files are never stamped):

```sh
python3 proto/ports/scout_batch_30/check.py
```

All 10 emitted C++17 binaries passed `-fsanitize=address,undefined` with zero errors. See `research/friction-backlog-batch-30-multi-topic.md` for full analysis.
