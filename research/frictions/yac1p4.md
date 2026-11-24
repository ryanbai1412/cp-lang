# yac1p4 — Fluke 2

## Bitset word ops recount on every call
- **Where:** `port.cp`, the inner loop's `union`, `shifted` and `shift_up`.
- **What felt wrong:** each in-place word op ends in `recount()` (another pass over every word)
  and `shifted` allocates, so the max case runs in 0.79 s against the accepted C++'s 0.42 s,
  close to the 1 s limit; the port had to be rewritten in place with a `below` swap to fit.
- **General fix:** keep `Bitset.len` lazy (count on demand, or track a dirty flag), and an
  in-place `or_shifted_from(o, k)` (`self |= o << k`) so a DP step needs no temporary.
- **Goal it hurts:** fast.
- **Severity:** medium.
