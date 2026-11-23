# Scout Batch 40 — Canadian and DMOJ Classics (`cheerio*`, `tle*`)

Five previously unported corpus problems from the `ecoo*`, `dwite*`, `vm7wc*`, `tle*`, `gfssoc*`, `mwc*`, `valentines*` and `cheerio*` prefix bucket, rethought in CP Lang against the locked v2 spec (`docs/one-page-cheat-sheet.md`). Findings are in `research/archive/frictions/friction-backlog-batch-40-canadian-dmoj.md`.

The batch was asked for ten problems. The bucket holds only ten codes in `corpus/submissions/info.json`, and five of them were not eligible:

- `cheerio1s3`, `cheerio1s4`, `dwite10c3p3` and `tle17c5p6` already have ports under `proto/ports/`.
- `dwite08c5p4` has no C++ submission. Its only accepted source (`3659678.js`) is JSFuck that prints five fixed numbers, so there is no algorithm to port.
- No code in the corpus starts with `ecoo`, `vm7wc`, `gfssoc`, `mwc` or `valentines`.

The batch stays inside its prefix partition rather than borrowing problems from another scout's bucket.

Each problem has two files:

- `<problem>.cp` is the v2-spec port: what the solution should look like once the cheat sheet is fully implemented. These files are **not compiled**.
- `today/<problem>.cp` is the same algorithm in the form `proto/cpc` builds now. These are the files that were run and checked.

## Problems

- `cheerio1s1` (`corpus/submissions/5337471.cpp`) — Cheerio Contest 1 S1. Drive past traffic lights, waiting out the red part of each cycle.
- `cheerio1s2` (`5337643.cpp`) — Cheerio Contest 1 S2. Sum of the `n` cheapest seats when seat `j` of row `i` costs `a * i + b * j`: a k-way merge on a min-heap.
- `cheerio1s5` (`5339756.cpp`) — Cheerio Contest 1 S5. Interval DP for the best score inside one block, then a second DP choosing disjoint blocks under a budget of `H` characters.
- `tle17c5p1` (`5034414.cpp`) — TLE '17 Contest 5 P1. Interactive: one guess, the judge answers with the difference.
- `tle18p5` (`5876770.cpp`) — TLE '18 P5. Longest strictly increasing subsequence with reconstruction. The judge supplies a header with the constant `N`, `get_item` and `print_num`; the original has no includes and does the whole DP in recursive templates.

All five were written from the accepted C++ (the corpus has no statements), not statement-first. The problem summaries above are read off the code.

## Verification

```
python3 proto/ports/scout_batch_40/check.py            # all five
python3 proto/ports/scout_batch_40/check.py cheerio1s5 --cases 50
python3 proto/ports/scout_batch_40/loc.py              # dual LOC table
```

`check.py` copies each `today/*.cp` to a temporary directory, builds it with `proto/cp.py build`, compiles the emitted C++ with ASan and UBSan, and compares it with the original submission on random inputs: 300 per problem (200 small, 100 medium), except `tle18p5` with 40. All 1,240 cases agree.

- `cheerio1s2` is also checked against sorting all `n * n` prices (for `n <= 300`).
- `tle18p5` is also checked independently: the printed sequence must be a strictly increasing subsequence of the input whose length is the O(n^2) optimum.
- `tle17c5p1` runs the port and the original each against a judge that answers `guess g` with `secret - g` and accepts only `answer secret`.

What this does not cover:

- The v2 files are checked by reading only.
- The originals are compiled with plain `-O1`, not sanitizers.
- `tle18p5`: the original needs `N` at compile time, so every case recompiles it against a generated header; that is why it gets 40 cases with `N <= 60`. The header (`constexpr` `N` and `get_item`, a `printf` `print_num`) is a reconstruction, not the judge's. The `today/` port reads `N` and the items from stdin instead of exposing grader hooks.
- `tle17c5p1`: the judge protocol (reply is `secret - guess`) is inferred from the one accepted submission.
- `cheerio1s1` and `cheerio1s5` have no independent brute force; they are compared with the original only. Input constraints are guesses (sorted distinct light positions below `k`; strings of `H` and `T`).
- No maximum-size inputs and no timing. Medium cases stay at `n <= 3000` (`cheerio1s5`: `n <= 70`).
