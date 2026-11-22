# noi09p4

Score: pending cold rescore (added dream `proto/ports/scout_batch_15/dream/noi09p4.cp` + `research/frictions/noi09p4.md`; `scout_batch_15/noi09p4.cp` now bounds `(n, m): (1..=20, 1..=30)`, uses `Row`/`Col`/`Cell.at`, derives the `protects` and `needs` edge directions in the header, and fixes `needs` to `(u: v, v: u)`; verified `cpl run` == `g++` on sample -> `25`, 18.7 KB, 0 warnings, 0 errors)
Dream: pending cold rescore (new dream using product domain `domain Cell = (Row, Col)` and length-prefixed `read()`)

Baseline review, cold, on main 2cca08a5. Two ports share the name: proto/ports/scout_batch_15/noi09p4.cp (27 lines) and proto/ports/scout_batch_18/noi09p4.cp (65 lines plus a brute). One of them should go; the review covers both. Both read as the editorial: peel the protection graph, then a max closure (batch_15 lines 22-27, batch_18 lines 65-70). No dream exists, so the score is capped at 3.

## Checklist failures

- batch_15: `n` and `m` unbounded (line 6); no `Row`/`Col` domains, so `cell(r, c)` takes raw `Int` (line 8).
- batch_15: `needs` (line 26) has `(u, v)` as protected-before-protector, while batch_18 (line 69) writes `(u: v, v: u)`; the edge direction isn't stated in either.
- batch_18: brute (lines 72-90) uses a `grew` flag loop (lines 82-87) and free `all(...)`/`sum(...)` (lines 85, 88-89), against §53. Withdrawn: §53.1 keeps the free generator form canonical.
- batch_18: `.to_idx` arithmetic (line 49, 80).

## Rewrites owed with today's language

- Keep one port; in batch_18's brute use `.all(...)`/`.sum(...)` methods and a fixpoint without the flag.

## Gaps that need the language

- WISH W17 (product domain `(Row, Col)` as an index) and W15 (length-prefixed per-cell lists in typed `read`) in batch_18 lines 46-52 are genuine; no research/frictions/noi09p4.md exists, so flag it.
