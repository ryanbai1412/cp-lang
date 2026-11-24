# yac1p4

Score: pending
Dream: pending

Follow-up: each DP step is now `swap(below, reach[open])` and then two
`reach[open].union_shifted(...)` (b3101637a), so no step allocates or builds a temporary. At
--max the native build takes 0.15 s, against 0.24 s before; all-Z n = 250 takes 0.31 s, against
0.43 s. The Bitset recount friction is fixed in std, so its friction file is removed.
