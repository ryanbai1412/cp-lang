# Scout Batch 44 — DMOPC 2020-2023 Classics (v2 Spec)

This batch scouts 10 previously unported problems from the strict `dmopc20*` / `dmopc21*` / `dmopc22*` / `dmopc23*` prefix bucket in `corpus/submissions/` against the locked CP Lang v2 specification (`docs/one-page-cheat-sheet.md`, commit `3723dca`).

## Corpus Prefix Note

`corpus/submissions/info.json` holds 42 problem codes in this bucket with an AC C/C++ submission, all of them `dmopc20*` or `dmopc21*` (there are no `dmopc22*` or `dmopc23*` submissions). Earlier batches already ported the P4-P6 tier, so only 18 codes were still unported. This batch takes the 10 hardest and most varied of those; they are P2-P4 problems, so the batch is honest about being mid-difficulty rather than "hard". The 8 left over are P1/P2 warm-ups plus `dmopc21c5p5`, a randomized heuristic construction whose output can only be checked with the problem's own validator.

## Layout

- `<code>.cp` — idiomatic CP Lang **v2 spec** ports (128 non-blank/non-comment lines total vs 832 raw C++ lines and 368 library-stripped C++ lines: **84.6% raw** and **65.2% library-stripped** reduction). These are design sketches against the cheat sheet and are not expected to run on today's prototype.
- `today/<code>.cp` — runnable ports for today's `proto/cp.py lib-run` interpreter and `stdlib/` (235 lines total: **71.8% raw** and **36.1% library-stripped** reduction).
- `check.py` — differential harness that compiles the 10 original AC C++ submissions with `g++ -O2 -std=c++17` and checks every `today/` port on randomized and edge-case inputs.

## Selected Problems (10)

- `dmopc21c8p2` (`4522631.cpp`) — parity-separated column sums: intersect allowed-sum intervals per parity class, then count fillings with `Mod` powers. Raw 103 / stripped 41 / v2 18 / today 34 LOC.
- `dmopc21c4p4` (`4131060.cpp`) — modular closed-form sum over binomials, factorials and inverse powers. Raw 77 / stripped 14 / v2 6 / today 12 LOC.
- `dmopc20c5p2` (`3591332.cpp`) — lattice geometry: the cells an `n x m` grid diagonal passes through. Raw 69 / stripped 30 / v2 9 / today 17 LOC.
- `dmopc21c9p2` (`4563973.cpp`) — string reduction with a stack that merges equal neighbours while a second string consumes its front. Raw 60 / stripped 25 / v2 14 / today 17 LOC.
- `dmopc21c5p3` (`4218397.cpp`) — prime-step game DP plus a constructive permutation with three hard-coded exceptions. Raw 122 / stripped 66 / v2 13 / today 31 LOC.
- `dmopc21c4p2` (`4125290.cpp`) — smallest-prime-factor sieve, divisor enumeration and an ascending greedy. Raw 95 / stripped 24 / v2 8 / today 22 LOC.
- `dmopc21c9p3` (`4564027.cpp`) — all-pairs BFS and a two-walker meeting argument per query. Raw 86 / stripped 52 / v2 12 / today 29 LOC.
- `dmopc21c7p2` (`4452915.cpp`) — block moves on an intrusive doubly linked list. Raw 72 / stripped 39 / v2 21 / today 27 LOC.
- `dmopc21c2p3` (`3946741.cpp`) — interactive randomized bit-prefix discovery (`@interactor` in v2, inline judge in `today/`). Raw 89 / stripped 53 / v2 23 (12 solver-only) / today 32 LOC.
- `dmopc21c6p2` (`4342045.cpp`) — run-length extraction and a top-`k` sum. Raw 59 / stripped 24 / v2 4 / today 14 LOC.

## Running Verification

```bash
python3 proto/ports/scout_batch_44/check.py                # 30 cases x 10 ports = 300 tests
python3 proto/ports/scout_batch_44/check.py dmopc21c9p3 60
```

All 10 ports pass 30/30 cases (300/300 total) against the compiled original C++ submissions. Two caveats:

- `dmopc21c2p3` is interactive and the corpus has no statement, so the judge (answer = distance between the guess and the hidden value in the tree where the parent of `x` is `x >> 1`) is inferred from the original's protocol. `check.py` drives the original C++ with that judge, and the original's own `assert(missed % 2 == 0)` holds throughout, which supports the inference. The port and the original must both pin down every hidden value in under 32,000 queries.
- `dmopc21c4p2` is tested with values up to 20,000 instead of the original's 5,000,000, because building the sieve in the tree-walking interpreter is too slow beyond that.

See `research/archive/frictions/friction-backlog-batch-44-dmopc-20-23.md` for the friction analysis.
