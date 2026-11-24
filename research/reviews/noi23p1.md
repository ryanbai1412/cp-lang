# noi23p1

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main 6a989896d: port, dream and friction report, not the C++. There is no dream (neither dream.cp nor a `## Dream` in the friction report), so the score is capped at 3 (research/reviews/README.md). One `IntervalSet` per compressed line, a Fenwick sweep for horizontal-vertical crossings and per-diagonal hits follow the header.

## Checklist failures

- Line 47 tags sweep events with `0 enter / 1 ask / 2 leave` integers whose order is the sort order, and line 58 uses `1 - what` as the Fenwick delta: an enum with that order says it (idiomatic-code/R5).
- Lines 66-69 test `x in 0..cols` and then build `Col.nth(x)` three times: a column found by `ord` arithmetic is checked and converted by hand (idiomatic-code/R6).
- Lines 74-76 sort, dedup and count `hits`; `hits.deduped().len` or a `Set<Col>` is the one spelling.

## Owed rewrites

- Write the dream.
- Name the event kinds with an enum.

## Language gaps

- None.
