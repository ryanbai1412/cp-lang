# ccc05s5

Score: 4/5
Dream: 3/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. An ordered multiset
of earlier scores, the count above each new one, and an exact average with `fixed`: the
editorial in four lines.

## Checklist failures
- `seen.len - seen.rank(s + 1)` (line 16) is the complement of a rank, which only works for
  integer keys; in the friction (`count_gt`), and the run is now there (E0203).
- idiomatic-code/R10: 0.07 s against the C++'s 0.04 s (1.75x, the friction's measurement)
  with no performance friction. Filed in the friction now.

## Owed rewrites
None today.

## Language gaps
- `count_lt/le/gt/ge` on `SortedBag`: in the friction, run added.
- 1.75x at `--max`: added to the friction (perf).

## Dream notes
`sum(1 + seen.count_gt(s) after seen.add(s) for s in scores)` hides the insertion inside the
sum (idiomatic-code/R7, caps at 3). Keep the query and the insertion as two statements.
