# Scout Batch 46 — DMOJ Contest Classics, Part 2 (`aac`, `yac`, `sac`)

Ten previously unported problems from the `aac*`, `yac*`, `wac*`, `sac*`, and `dmpg*` prefix bucket in `corpus/`, with no overlap with `scout_batch_39` or any other `proto/ports/`, `scout_batch_*`, or `blind_critic_*` port.

The bucket is nearly exhausted. Every `wac*` and `dmpg*` problem with an AC C++ submission is already ported, so this batch has none. The hardest unported problem left, `sac22cc4p5` (598 lines, interactive merge-insertion sort with a time-seeded RNG), was skipped because its query sequence is not reproducible and so cannot be diffed against a port. The ten below are the hardest of what remains; several are short.

Each problem has two files:

- `<prob>.cp` — idiomatic CP Lang against the locked **v2 spec** (`docs/one-page-cheat-sheet.md`, `3723dca`). These are design sketches; the prototype does not compile them.
- `today/<prob>.cp` — runnable version built with `proto/cp.py build` and checked against the original AC C++ submission.

## Problems and dual LOC

Non-blank, non-comment lines. `raw` is the whole C++ file. `stripped` drops the shared debug/macro header and reusable library code: the `SegTree` class in `sac22cc2p5`, the stream-printing templates in `yac6p5`, and the author's local `stress()` driver in `yac1p3`. `main()` and everything problem-specific stays, including the running-median `DS` struct in `yac6p5`.

- `aac4p4` (`4140376.cpp`, constructive 0/1 matrix with a cell-count window): 148 raw / 123 stripped -> **32 v2** (-78.4% raw, -74.0% stripped), 64 today.
- `aac5p4` (`4239900.cpp`, closed-form parity answer): 61 raw / 36 stripped -> **10 v2** (-83.6% raw, -72.2% stripped), 14 today.
- `yac1p2` (`4376253.cpp`, tree path counting with a 3x3 colour-count DP per node): 134 raw / 109 stripped -> **19 v2** (-85.8% raw, -82.6% stripped), 43 today.
- `yac1p3` (`4380584.cpp`, interactive grid game won by mirroring through the centre): 130 raw / 46 stripped -> **9 v2** (-93.1% raw, -80.4% stripped), 16 today.
- `yac6p4` (`5149666.cpp`, nim-sum over a precomputed Grundy table): 57 raw / 32 stripped -> **12 v2** (-78.9% raw, -62.5% stripped), 18 today.
- `yac6p5` (`5149823.cpp`, two-median split per axis; rethought from two-multiset running medians to sort + prefix sums): 176 raw / 137 stripped -> **13 v2** (-92.6% raw, -90.5% stripped), 38 today.
- `yac7p2` (`5705152.cpp`, constructive numbered walk over a grid): 136 raw / 115 stripped -> **50 v2** (-63.2% raw, -56.5% stripped), 84 today.
- `sac22cc2p3` (`4121049.cpp`, best root-to-leaf choice over a fork tree): 62 raw / 37 stripped -> **11 v2** (-82.3% raw, -70.3% stripped), 36 today.
- `sac22cc2p5` (`4121099.cpp`, point assign + same-parity range sum): 210 raw / 49 stripped -> **10 v2** (-95.2% raw, -79.6% stripped), 32 today.
- `sac22cc3p3` (`4331598.cpp`, digit-by-digit suffix sort passes; rethought from padded strings to `a % 10^d`): 63 raw / 38 stripped -> **8 v2** (-87.3% raw, -78.9% stripped), 18 today.

**Total:** 1,177 C++ raw / 722 C++ stripped -> **174 CP Lang v2** (-85.2% vs raw, -75.9% vs stripped) and **363 runnable `today/`** (-69.2% vs raw, -49.7% vs stripped).

## Verification

```bash
python3 proto/ports/scout_batch_46/check.py
```

This builds each original and each `today/` port (emitted C++17 under ASan + UBSan) and requires identical output on 150 random small/medium inputs per problem, seed 46. All ten pass.

What that does and does not cover:

- `yac1p3` is interactive. The original plays a seeded random opponent through a Python judge that checks every move is legal. The port is not played live, because `cp.py build` output is only flushed at exit; instead it is fed the judge's side of that game and must print the original's moves exactly.
- `aac4p4` and `yac7p2` are constructive, but both ports follow the original's construction, so they are compared for exact equality rather than through a validator.
- `sac22cc2p3`'s input shape was inferred from the original's reader (`2^k - 1` fork lines over `2^(k+1) - 1` nodes), not from the statement.
- Inputs are small and medium only. No max-N timing or memory runs were done, and branch coverage of the generators was not measured.

Friction found in the v2 spec and the prototype is in `research/friction-backlog-batch-46-dmoj-contests-2.md`.
