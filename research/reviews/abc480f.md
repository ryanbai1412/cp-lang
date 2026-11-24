# abc480f

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against the ideal under `## Dream` in `research/frictions/abc480f.md`
(there is no `dream.cp`), without the C++. The header derives why cutting at multiples of S / M
is optimal and that the answer is M minus the most prefixes hitting a multiple. `rem` and
`saved` are tables over `Subset<I>`.

## Checklist failures
- idiomatic-code/R2: `rem[s]` is the subset sum mod t, written as a recurrence on `s.first()`.
  It is correct and O(2^N), but the reader has to recognise the sum; the friction routes a std
  subset-sum table.

## Owed rewrites
None that today's language closes.

## Language gaps
- Subset-sum table over `Subset<I>` (in the friction).

## Dream notes
The friction's dream invents a `dp(...)` self-referential table builder; a plain
`for s in Subset<I>` loop states the same order. 4/5.
