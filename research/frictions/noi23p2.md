# Friction Report: noi23p2

No friction left. The k-bit mask of pending deadlines is a `Subset<Slot>` over `domain Slot =
0..k` inside the test loop, and the DP table is `[Slot.subsets]Z`. The hand-written popcount
table, the `x & (-x)` loop over set bits, the shift-and-xor that drops the top bit and the
separate `k == 0` product are now `wait.len`, `for b in wait`, `wait.shifted(1)` and the
`Slot.last` / `Slot.first` optionals. The DP's single state for k = 0 is the product.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and the package's 6 cases pass. On the max case the port takes 0.29 s, the v0.2 port
0.19 s and the accepted C++ 0.18 s (limit 0.5 s), all with the same output.

## 1. The subset DP is 1.6x the C++
- **Where:** `var w = (2 * c - 1) * ways[later].v` ... `next[wait] = Z.new(w)`, the inner loop over `Slot.subsets`.
- **What felt wrong:** even with the residues summed as raw `Int` and reduced once, the max case runs in
  0.29 s against the v0.2 port's 0.19 s and the accepted C++'s 0.18 s (measurement above), over the
  1.1x line, and the plain `Z` sum the header wants is given up for speed.
- **Ideal code:** `next[wait] = (2 * c - 1) * ways[later] + ... + sum(ways[wait.without(b).shifted(1)] for b in wait)`.
- **General fix:** profile the `Subset<Slot>` ops (`shifted`, `without`, `with`) and `[Slot.subsets]Z`
  indexing in this loop until they compile to the mask ops the v0.2 port wrote, and let `Mod<P>` sums
  defer their reduction so the plain spelling is the fast one.
- **Owner:** perf. **Severity:** medium.
