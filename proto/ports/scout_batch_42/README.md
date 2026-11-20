# Scout Batch 42 — Hard Mixed Corpus Classics

Ten unported problems drawn strictly from the Batch 42 prefix bucket (`cpc*`, `hci*`, `mcco*`, `mmcc*`, `occ*`, `pacnw*`, `rte*`, `seed*`, `thicc*`, `tsoc*`, `wc*`, `year*`, `bbc*`, `lkp*`, `mccc*`), ported against the locked CP Lang v2 spec (`docs/one-page-cheat-sheet.md` at `3723dca`) and verified differentially against the original C++ submissions in `corpus/submissions/`.

## Prefix Bucket Coverage

Across all 836 unique problem codes in `corpus/submissions/info.json`, 22 codes match the 15 Batch 42 prefixes (the corpus contains no submissions for `cpc*`, `pacnw*`, `rte*`, `seed*`, `thicc*`, `tsoc*`, or `year*`). Five were already ported in earlier batches (`bbc09d`, `lkp18c1p4`, `mcco17d1p2`, `mmcc14p3`, `wc18c3s2`) and one has only a compile-error stub (`hci16ulamspiral`), leaving 16 unported problems (`occ19g4` plus 15 `mccc*` problems). From those 16, this batch selects the 10 hardest and most algorithmically diverse problems spanning two-pass sweeps with ordered sets, 2D rotated-Manhattan CDQ divide-and-conquer, bitwise XOR-prefix tries, offline rank counting, 2D group-stage tables with enum tags, simultaneous FIFO queue ticks, `Subset<V>` maximum independent set, greedy wildcard string construction, character-domain frequency optimization, and prefix-sum equilibrium splits.

## Layout

- `<problem>.cp`: idiomatic CP Lang v2 solutions following `docs/one-page-cheat-sheet.md` (`3723dca`), with inline `// FRICTION MC#` notes where the v2 spec leaves a gap or collision.
- `today/<problem>.cp`: runnable versions executed by `proto/cp.py lib-run` today.
- `check.py`: differential test harness compiling all 10 original C++ submissions with `g++ -O2 -std=c++17` and running 30 small/medium randomized + edge cases per problem (300 differential tests), plus 20 extra brute-force edge cases covering bugs in the original C++ submissions for `mccc6s3` and `occ19g4`.
- `loc.py`: dual LOC counter reporting raw C++ LOC, library-stripped C++ LOC, v2 LOC, and runnable `today/` LOC.

## Summary of Ports and Dual LOC

All counts are non-blank, non-comment lines (`python3 proto/ports/scout_batch_42/loc.py`). Across all 10 problems, raw C++ totals 807 lines and library-stripped C++ totals 369 lines; CP Lang v2 totals 234 lines (**-71.0%** vs raw, **-36.6%** vs stripped), and runnable `today/` totals 381 lines (**-52.8%** vs raw, **+3.3%** vs stripped). Excluding `occ19g4` — where the port replaces a TLE sparse 2D `gp_hash_table` BIT with a full exact two-pass CDQ divide-and-conquer + 1D Fenwick solution — the remaining 9 problems drop from 693 raw / 324 stripped lines to 158 v2 lines (**-77.2%** vs raw, **-51.2%** vs stripped).

- `mccc6s3` (`6103125.cpp`): 141 raw / 66 stripped -> 40 v2 (-71.6% / -39.4%), 89 today. Partition balancing with one optional element move; forward/reverse sweep over `Set<Int>` + `HashMap<Int, Deque<Pos>>`. Also catches two real bugs in `6103125.cpp` (a `next_ind[m]` stack overflow when `max(a) < n` and a missed odd-`diff` tie candidate).
- `occ19g4` (`3578469.cpp`): 114 raw / 45 stripped -> 76 v2 (-33.3% / +68.9%), 105 today. Rotated-Manhattan 2D range counting rethought as two-pass CDQ divide-and-conquer + coordinate-compressed 1D `Fenwick`, eliminating both the corpus submission's TLE and its negative-index BIT infinite loop.
- `mccc5j5` (`5333196.cpp`): 78 raw / 47 stripped -> 20 v2 (-74.4% / -57.4%), 34 today. XOR-prefix range maximum `(i ^ m) <= k` via iterative complete binary trie sized from `bit_len`.
- `mccc5j3` (`5333116.cpp`): 67 raw / 30 stripped -> 16 v2 (-76.1% / -46.7%), 29 today. Distant pair-sum counting (`a[i] + b[j] == m`, `|i - j| >= k`), replacing `pb_ds` order-statistic trees with sorted bucket lists and `first_ge`.
- `mccc4j4` (`4264934.cpp`): 101 raw / 46 stripped -> 21 v2 (-79.2% / -54.3%), 28 today. Group-stage ranking with `@tags` enum outcomes, 2D `[Group, Slot]Team` storage, and per-row mixed-order sorting.
- `mccc3j4` (`4223440.cpp`): 66 raw / 31 stripped -> 14 v2 (-78.8% / -54.8%), 21 today. Simultaneous tick FIFO queue simulation using `[Player]Deque<Player>` and `[Player]?Player`.
- `mccc3j5` (`4223449.cpp`): 59 raw / 24 stripped -> 11 v2 (-81.4% / -54.2%), 18 today. Maximum independent set over `n <= 20` vertices using `Subset<V>` instead of raw bit shifts and `goto`.
- `mccc5j4` (`5333145.cpp`): 61 raw / 30 stripped -> 17 v2 (-72.1% / -43.3%), 20 today. Lexicographically smallest wildcard string fill rethought as a single forward greedy pass without a stack.
- `mccc3j3` (`4223424.cpp`): 68 raw / 33 stripped -> 12 v2 (-82.4% / -63.6%), 23 today. Maximum character-frequency dot product after one replacement, simplified from the original's O(26^3) triple loop to O(n + m + 26) algebra over `domain Letter = 'A'..='Z'`.
- `mccc4j3` (`4264907.cpp`): 52 raw / 17 stripped -> 7 v2 (-86.5% / -58.8%), 14 today. First 1-based prefix-sum equilibrium split using `prefix(a)`, `first(...)`, and `print(ans, none: ...)`.

## Verification

Run the differential harness and dual LOC report from the repository root:

```bash
python3 proto/ports/scout_batch_42/check.py
python3 proto/ports/scout_batch_42/loc.py
```

Latest run: all 10 ports pass 30/30 randomized differential cases (300/300 total), plus 10/10 brute-only small-value cases on `mccc6s3` and 10/10 brute-only negative/extreme-coordinate cases on `occ19g4`. See `research/friction-backlog-batch-42-mixed-classics.md` for the full friction analysis.
