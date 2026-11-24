# egoi22p5

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against the friction's `## Dream`. Ascending counts with the top `c` as the tail, subtract, then restore order by merging the two sorted runs (lines 16-28): the header's argument.

## Checklist failures
- idiomatic-code/R3: lines 25-28 find the overlap window with two `first_true` searches and copy `merge_sorted(...).collect()` back by index. The friction measures why (a full `merge_sorted` per service is 2.2 s, TLE) and reports the missing in-place run merge, so this is a measured deviation.

## Owed rewrites
None today.

## Language gaps
- No in-place merge of adjacent sorted runs: `free.merge_runs(cut)` is `error[E0203]: a List<Int> has no method merge_runs` on a fresh `cpl`; repro now in the friction.

## Dream notes
`free.merge_runs(cut)` names the operation; the two-line fragment keeps the port's reads and header.
