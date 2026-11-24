# yac6p6 review

Score: pending
Dream: pending

Cold review: rewritten for the 1000 ms limit (the old 3-cell pivot spacing ran about 1.7 s at
--max; this runs 0.48 s against the AC's 0.64 s). Pivots sit every L = 16 anti-diagonals, so a
query either walks its small subgrid directly or is bucketed at the diagonal cells inside it;
one backward walk per pivot starts at the bucket's furthest-back start and one forward walk ends
at its furthest end. Rows, columns and queries are domains, reads are typed, the answer table is
`?Int`, and the small-query DP is the same `walk_back` as the pivot DP, so there is one
recurrence per direction instead of three copies.
