# Friction Report: noi15p5

No friction left: ranks in suffix order get their own `Rank` domain (the v0.2 port reused
`Glass`) and the bounds are types. The adjacent pairs are sorted by LCP, descending, and a
`next` pointer walks them as r falls (port lines 40-46); an earlier note described buckets
`meet_at[r]` that the port does not have.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 3e5, all `a`) 0.09 s vs C++ 0.09 s.
