# Friction Report: noi18p3

No friction left: the hand-rolled persistent segment tree (`left`/`right` lists, `single`,
`union`, `any_in` with explicit node bounds) is std `MergeSegs<Place>`: `singleton` per end,
`merge` up the suffix links (both inputs stay valid), and the read-only `any_in` per step,
added to std for this port. Query bounds are typed reads.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (|s| = 5e5, q = 1e5, sum |t| = 1e6)
1.68 s vs C++ 1.99 s, same output.
