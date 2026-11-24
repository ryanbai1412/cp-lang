# Friction Report: noi15p5

No friction left: ranks in suffix order get their own `Rank` domain (the v0.2 port reused
`Glass`), the bounds are types, and the adjacent pairs are bucketed by LCP so each r unites
`meet_at[r]` instead of moving a pointer through a sorted list.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 3e5, all `a`) 0.09 s vs C++ 0.09 s.
