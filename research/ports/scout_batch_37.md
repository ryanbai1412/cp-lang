# Scout Batch 37 — Hard IOI, NOI and NOIP Contest Classics

Ten previously unported corpus problems from the `ioi*`, `noi*` and `noip*` prefix bucket, rethought in CP Lang against the locked v2 spec (`docs/one-page-cheat-sheet.md`). Findings are in `research/archive/frictions/friction-backlog-batch-37-ioi-noi.md`.

Each problem has two files:

- `<problem>.cp` is the v2-spec port: what the solution should look like once the cheat sheet is fully implemented. These files are **not compiled**; several use v2 features `proto/cpc` does not have yet (`@grader`, `.step(k)`, `p.adj4`, `Deque`, `while ... and let`).
- `today/<problem>.cp` is the same algorithm in the form `proto/cpc` builds now. These are the files that were run and checked.

## Problems

- `noi10p1` (`corpus/submissions/5670455.cpp`) — NOI 2010 Energy Collection. Count pairs by exact gcd with a harmonic sieve.
- `ioi15p1` (`5723345.cpp`) — IOI 2015 Boxes with Souvenirs. Prefix and suffix trip DP on a circle with at most one full lap. Grader function `delivery`.
- `noi14p5` (`3601452.cpp`) — NOI 2014 Random Number Generator. Seeded shuffle of an `n x m` board, then a greedy lexicographically smallest right/down path with per-row column windows.
- `noip18p4` (`4004140.cpp`) — NOIP 2018 Travel. Smallest DFS order on a tree or a unicyclic graph, trying each edge as the unused one.
- `ioi09p5` (`5106592.cpp`) — IOI 2009 Garage. Event simulation with a min-heap of free spots and a FIFO queue.
- `noip06p3` (`5165783.cpp`) — NOIP 2006 Job Scheduling. Earliest-gap insertion on per-machine timelines.
- `noip10p4` (`5205786.cpp`) — NOIP 2010 Water Diversion. Flood fill from the top row, memoized bottom-row interval per cell, greedy interval cover.
- `ioi20p2` (`4724578.cpp`) — IOI 2020 Connecting Supertrees. Stars for `p == 1`, one ring per `p == 2` group, then exact validation. Grader function `construct`.
- `ioi12p5` (`4951033.cpp`) — IOI 2012 Last Supper. Belady eviction with one advice bit per stay. Grader functions `ComputeAdvice` and `Assist`.
- `noip04p4` (`5117252.cpp`) — NOIP 2004 Alphametic Puzzle. Column-by-column backtracking in base `n` with pruning on fully known columns.

All ten were written from the accepted C++ (the corpus has no statements), not statement-first.

## Verification

```
python3 proto/ports/scout_batch_37/check.py            # all ten, 300 cases each
python3 proto/ports/scout_batch_37/check.py ioi09p5 --cases 50
python3 proto/ports/scout_batch_37/loc.py              # dual LOC table
```

`check.py` copies each `today/*.cp` to a temporary directory, builds it with `proto/cp.py build`, compiles the emitted C++ with ASan and UBSan, and compares its output token by token with the original submission on 300 random inputs per problem (200 small, 100 medium). All 3,000 cases agree.

What this does not cover:

- The v2 files are checked by reading only.
- The originals are compiled with plain `-O1`, not sanitizers: `4004140.cpp` trips an ASan stack-use-after-scope report on its own.
- The three grader problems run behind a small stdin/stdout `main` appended to the original, and the `today/` ports read the same stdin instead of exposing grader functions. `ioi20p2` and `ioi12p5` accept many answers; the comparison is against this one original, which the ports follow step for step, not against an official checker.
- No maximum-size inputs and no timing. Medium cases stay in the hundreds to low thousands.
- `noi14p5` declares `a <= 300` so that `a * x * x` provably fits `Int`; the generator stays inside that bound.

## LOC

Non-blank, non-comment lines. "Stripped" drops the contest template at the top of each C++ file and the stream setup lines.

```
problem      raw stripped   v2 today
noi10p1       66       42    9    13
ioi15p1       48       26    7    11
noi14p5      101       73   29    35
noip18p4     101       72   23    32
ioi09p5       70       42   19    26
noip06p3      94       66   20    24
noip10p4     132      104   30    39
ioi20p2      116       92   30    44
ioi12p5      101       77   42    42
noip04p4     101       73   34    37
total        930      667  243   303
```

The v2 ports are 74% shorter than the raw C++ and 64% shorter than the stripped C++; the buildable forms are 67% and 55% shorter.
