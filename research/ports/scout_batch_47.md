# Scout Batch 47 — Hard CCC & CCO Classics Part 2 (`ccc*`, `cco*`)

Ten previously unported accepted DMOJ CCC and CCO submissions ported idiomatically to the locked CP Lang v2 specification (`docs/one-page-cheat-sheet.md` at `3723dca`), paired with runnable `today/` implementations verified directly against the original C++ submissions under `g++ -std=c++17 -O2 -fsanitize=address,undefined -fno-sanitize-recover=all`.

## Problems

- **`cco23p5.cp`** (orig `corpus/submissions/5580402.cpp`, 419 raw / 392 stripped C++ LOC -> 137 v2 / 229 today): Weighted tree k-jump path (`k in {1, 2, 3}`) with 4-state subtree DP, top-k distinct child matching, constructive walk recovery, and alternating pre/post-order DFS.
- **`cco24p5.cp`** (orig `corpus/submissions/6435483.cpp`, 257 raw / 72 stripped C++ LOC -> 51 v2 / 92 today): Parity-alternating subarray partition DP mod `1_000_003` using two `LazySegTree<I.cuts, Node>` trees maintaining `(min_bad, ways)` over split boundaries.
- **`cco24p4.cp`** (orig `corpus/submissions/6435484.cpp`, 174 raw / 79 stripped C++ LOC -> 38 v2 / 56 today): Tree rendezvous schedule construction on `n = 100` trees using tree centering (`eccentricity <= 50`) and alternating climb bursts.
- **`cco24p2.cp`** (orig `corpus/submissions/6435481.cpp`, 134 raw / 90 stripped C++ LOC -> 48 v2 / 74 today): Minimum stack-sorting partition (Pizza Party) combining multiset validation, binary reversal check, and reverse patience sorting / Dilworth LIS.
- **`ccc20s2hard.cp`** (orig `corpus/submissions/4324075.cpp`, 112 raw / 47 stripped C++ LOC -> 22 v2 / 51 today): Tight-memory `4000 x 4000` Escape Room reachability using `@grader`, flat CSR `Graph.directed`, and `Bitset<Val>`.
- **`cco01p4io.cp`** (orig `corpus/submissions/5034487.cpp`, 85 raw / 55 stripped C++ LOC -> 28 v2 sol / 50 with `@interactor` / 36 today): Interactive two-egg drop with `@memo` returning `(drops, step)`, `ask(...)` queries, and a co-located `@interactor`.
- **`cco07p5.cp`** (orig `corpus/submissions/5120565.cpp`, 83 raw / 45 stripped C++ LOC -> 31 v2 sol / 47 with `@interactor` / 32 today): Interactive moving-UFO ring search using randomized window probing, cyclic domain `.cyc()`, median candidate halving, and a co-located `@interactor`.
- **`ccc18s2.cp`** (orig `corpus/submissions/4329456.cpp`, 86 raw / 51 stripped C++ LOC -> 13 v2 / 32 today): `n x n` sunflower table rotation using a square domain `N`, `.row`/`.col` `.adjacent` checks, and `g.transposed.flipped`.
- **`ccc22s2.cp`** (orig `corpus/submissions/4371343.cpp`, 62 raw / 35 stripped C++ LOC -> 15 v2 / 31 today): Student triad constraint counting with `HashMap<Str, Group>` and `count(...)` comprehensions.
- **`cco04p1.cp`** (orig `corpus/submissions/5430016.cpp`, 62 raw / 31 stripped C++ LOC -> 17 v2 / 29 today): Letter-tile word scoring over ASCII byte arrays.

## Verification

Run the differential, constructive, and live interactive-pipe verification suite:

```bash
python3 proto/ports/scout_batch_47/verify.py
```

All 10/10 runnable `today/` ports compile cleanly via `proto/cp.py build` (on temporary copies) + `g++ -std=c++17 -O2 -fsanitize=address,undefined -fno-sanitize-recover=all` and pass 531 differential, constructive, and live interactive test runs against their original accepted C++ submissions.

See `research/archive/frictions/friction-backlog-batch-47-ccc-cco-2.md` for full dual-LOC analysis and v2 design friction findings.
