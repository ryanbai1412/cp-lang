# Scout Batch 41 — Hard Olympiad & NCCC Classics (`oly*`, `nccc*`)

Ten previously unported corpus problems from the `oly*`, `cerc*`, `ecna*`, `icpc*`, `naq*`, `nac*`, `waterloo*`, `mcipc*`, `utso*`, and `nccc*` prefix bucket, rethought in CP Lang against the locked v2 spec (`docs/one-page-cheat-sheet.md`, commit `3723dca`). Findings are in `research/friction-backlog-batch-41-oly-icpc.md`.

**Bucket coverage note:** `corpus/submissions/info.json` contains zero problems for `cerc*`, `ecna*`, `icpc*`, `naq*`, `nac*`, `waterloo*`, and `mcipc*`, and its only `utso*` problem (`utso15p5`) is already ported. Across `oly*` and `nccc*`, there are exactly 10 unported AC submissions in the entire corpus — 7 in C++17 and 3 in Java (`nccc8s1`, `nccc8s2`, `nccc8s4`) — so this batch covers all 10 with zero overlap against parallel scouts.

Each problem has two files:

- `<problem>.cp` is the v2-spec port written directly against `docs/one-page-cheat-sheet.md`. These files are **not compiled**; each hits one or more features not yet in `proto/cpc` (struct field defaults, filtered `for ... if`, tuple `read()`, `Str` operations, `Mod<P>`, `Subset<Q>`, `popcount`).
- `today/<problem>.cp` is the same algorithm in the runnable form accepted today. Eight build to C++17 via `proto/cpc` and run under ASan + UBSan; two unknown-length `Str` problems (`nccc8s1`, `nccc8s2`) run under `proto/cp.py lib-run`.

## Problems

- `oly20practice139` (`corpus/submissions/6415690.cpp`) — Two-option pricing revenue maximization. Coordinate-compressed y-values split into sqrt-blocks, each maintaining a monotonic upper Convex Hull Trick swept over sorted x-events.
- `oly23practice5` (`6415646.cpp`) — Multitest alternating-sequence minimum edits. Maximal alternating-run penalties in a hash map, even/odd frequency counts, and sorted-order best non-conflicting pair lookup (replacing the original's `multiset` + conflict graph).
- `nccc9s4` (`4145494.cpp`) — Range mode queries (ties broken by largest value). Heavy-light split: per-value prefix sums for `<= 800` distinct values, Rollback Mo's algorithm otherwise.
- `nccc9s1` (`4051682.cpp`) — Multitest parity and count feasibility check for splitting `a` 1s, `b` 2s, and `c` 3s.
- `nccc9s2` (`4051671.cpp`) — Minimum adjacent swaps to move all `n` `'I'` characters in a `2n`-char string to even positions.
- `nccc9s3` (`4051697.cpp`) — Prepend `'U'` to a `2n`-char `'U'/'I'` string and take `max(#"UU", #"II")`.
- `nccc10s1` (`4214147.cpp`) — Maximize minimum Hamming distance over all `2^k` binary answer keys against `n` `T`/`F` keys.
- `nccc8s1` (`3381111.java`) — Sorted deduplicated edit-distance-1 neighborhood (substitute, delete, insert) of a string over a given alphabet.
- `nccc8s2` (`3381135.java`) — Count distinct-letter subsequences modulo `10^9 + 7`.
- `nccc8s4` (`3381215.java`) — Directed cycle detection over tile edge labels in `-26..=26` (`x -> -y`).

All ten were written from the accepted corpus submissions (the corpus has no problem statements).

## Verification

```
python3 proto/ports/scout_batch_41/check.py             # all ten (2,520 cases total)
python3 proto/ports/scout_batch_41/check.py nccc9s4 --cases 50
python3 proto/ports/scout_batch_41/loc.py               # dual LOC summary
```

`check.py` verifies all eight `proto/cpc` ports (compiled with `-fsanitize=address,undefined -fno-sanitize-recover=all`) on 300 random inputs each (200 small, 100 medium) and both `lib-run` ports (`nccc8s1`, `nccc8s2`) on 60 random inputs each (40 small, 20 medium) against the original C++ (`g++ -O1`) or Java (`javac` + `java`) submission. All 2,520 cases agree with zero sanitizer errors.

Limits of this check:

- The top-level v2 files are verified by reading only (and probed against `proto/cpc` to capture exact prototype diagnostics).
- `nccc9s1`'s generator keeps `a, b, c <= 2e8` so the original C++'s 32-bit `int sum = a + b * 2 + c * 3` does not overflow.
- `nccc9s4`'s medium generator explicitly forces `> 800` distinct values on 60% of medium cases so both the prefix-sum branch and the Rollback Mo branch are exercised; no max-N (`10^5`) timing benchmarks were run.
- `nccc8s1` compares stdout line-by-line (preserving blank lines when deleting the sole character of a length-1 string).

## Dual LOC

Non-blank, non-comment lines counted by `loc.py`:

```
problem              raw stripped   v2 today
oly20practice139     167      142   55    91
oly23practice5       134      109   23    42
nccc9s4              166      123   53    95
nccc9s1               63       32    4     8
nccc9s2               58       27    4     9
nccc9s3               56       25    6     9
nccc10s1              56       28    9    25
nccc8s1               55       32   11    32
nccc8s2               37       13    7     9
nccc8s4               68       44   13    26
total                860      575  185   346
```

The v2 ports are 78% shorter than raw originals and 68% shorter than template-stripped originals (65% shorter even after also excluding the 41 lines of unused debug `operator<<` and dead brute-force helper left inside `6415646.cpp`). The runnable `today/` forms are 60% shorter than raw and 40% shorter than stripped.
