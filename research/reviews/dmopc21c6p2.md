# dmopc21c6p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

The zero runs between ones, sorted longest first; the answer sums the k longest.

## Checklist failures

- `runs.sort(by: x => desc(x))` then `runs[0..min(k, runs.len)].sum()` is a top-k sum by sorting and clamping a slice by hand.
