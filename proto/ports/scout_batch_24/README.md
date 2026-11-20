# Scout Batch 24: Hard Segment Trees, Lazy Propagation, Persistent/Implicit DS, Mo's & Offline Queries

This batch ports 10 unported, hard data-structure and offline-query problems from `corpus/submissions/` using the statement-first workflow:

1. Fetch problem metadata from `dmoj.ca/api/v2/problem/<code>` (when reachable) and reconstruct each problem from its published contest specification without reading the corpus C++ first.
2. Write an idiomatic CP Lang solution at `proto/ports/scout_batch_24/<code>.cp`, recording every language or standard-library friction point inline as `WISH A1`–`WISH A23`.
3. Inspect the original AC C++ submission in `corpus/submissions/` to verify exact I/O and compare algorithmic choices (9/10 matched I/O on the first pass; `cco18p5` had plain rather than XOR-encrypted updates and was adjusted after checking `4801867.cpp`).
4. Write a runnable port under `proto/ports/scout_batch_24/today/<code>.cp` executing via `python3 proto/cp.py lib-run` and differentially test all 10 against the compiled C++ originals via `proto/ports/scout_batch_24/check.py` (200/200 randomized + edge-case tests passing).

Full design analysis and concrete code snippets live in `research/friction-backlog-batch-24-advanced-ds.md`.

## Problems & Dual LOC Summary

Across all 10 problems, non-blank/non-comment LOC dropped from **2,549 raw C++ LOC** to **901 ideal CP Lang LOC (-64.7%)** and **875 runnable `today/` LOC (-65.7%)**. Against library-stripped C++ (excluding debug templates, fast-IO wrappers, and hand-rolled standard segment trees), LOC dropped from **1,351 stripped C++ LOC** to **901 ideal CP Lang LOC (-33.3%, including `@brute` specs)** and **875 `today/` LOC (-35.2%)**.

- **`ioi07p3`** (IOI '07 P3 — Sails, 25pt; `5120005.cpp`): Lazy min-range segment tree with prefix binary search (`max_right`). 318 raw / 48 stripped C++ LOC vs 54 ideal / 47 `today/` CP Lang LOC.
- **`noi07p4`** (NOI '07 P4 — Necklace Factory, 20pt; `5032746.cpp`): Cyclic orientation transform + custom non-commutative `Run` monoid and range-paint lazy segment tree. 299 raw / 103 stripped C++ LOC vs 71 ideal / 68 `today/` CP Lang LOC.
- **`noip22p4`** (NOIP '22 P4 — Contest, 30pt; `5445685.cpp`): Offline right-endpoint sweep + monotonic stacks + `U64` wrapping historical-sum lazy segment tree (5-field node, 6-field tag). 313 raw / 191 stripped C++ LOC vs 86 ideal / 96 `today/` CP Lang LOC.
- **`noi20p2`** (NOI '20 P2 — Destiny, 35pt; `5655455.cpp`): Tree DP with implicit segment-tree merging over `Pool<Node>` + `Node.Id` and `Mod<998244353>`. 335 raw / 162 stripped C++ LOC vs 89 ideal / 93 `today/` CP Lang LOC.
- **`noi05p2`** (NOI '05 P2 — Maintaining a Sequence, 30pt; `7249409.cpp`): Implicit treap over `Pool<Node>` with free-list recycling under a 64 MB memory limit, Kadane max-subarray aggregate, and dual reverse/assign tags. 202 raw / 183 stripped C++ LOC vs 185 ideal / 191 `today/` CP Lang LOC.
- **`cco18p5`** (CCO '18 P5 — Boring Lectures, 25pt; `4801867.cpp`): Online dual-`SegTree` local-peak invariant (`ArgMax` + `Max`, $O((N+Q) \log N)$), contrasted with a 153-line offline time segment tree + rollback max segment tree in C++. 183 raw / 153 stripped C++ LOC vs 68 ideal (including `@brute`) / 51 `today/` CP Lang LOC.
- **`ioi13p3io`** (IOI '13 P3 — Wombats, 40pt; `4722412.cpp`): Block segment tree of `C x C` Monge min-plus matrices with Knuth-accelerated $O(C^2)$ combine. 184 raw / 136 stripped C++ LOC vs 94 ideal / 105 `today/` CP Lang LOC.
- **`dmpg15g6`** (DMPG '15 G6 — Yokan, 17pt; `4253124.cpp`): Deterministic Rollback Mo's algorithm using `Journaled` + nested `undo` (contrasted with Misra-Gries Heavy-Hitter SegTree in C++). 298 raw / 101 stripped C++ LOC vs 83 ideal / 97 `today/` CP Lang LOC.
- **`ceoi22p1`** (CEOI '22 P1 — Abracadabra, 20pt; `4728849.cpp`): Offline query sweep by shuffle count + monotonic-stack run splitting + `Fenwick.lower_bound`. 178 raw / 65 stripped C++ LOC vs 93 ideal (including `@brute`) / 65 `today/` CP Lang LOC.
- **`cco16p5`** (CCO '16 P5 — Zombie Apocalypse, 15pt; `4580695.cpp`): 2D Klee's Measure Problem (`union_area(Q) - union_area(Q-1)`) with coordinate-compressed sweep-line and tag-free Cover-Count segment tree (contrasted with 209 lines of boundary-segment geometry in C++). 239 raw / 209 stripped C++ LOC vs 78 ideal (including `@brute`) / 62 `today/` CP Lang LOC.

## Running Differential Tests

```bash
python3 proto/ports/scout_batch_24/check.py
```

Every port runs 20 randomized and edge-case tests (200 total) against the compiled original AC C++ submission.
