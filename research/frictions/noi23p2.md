# Friction Report: noi23p2

No friction left. The k-bit mask of pending deadlines is a `Subset<Slot>` over `domain Slot =
0..k` inside the test loop, and the DP table is `[Slot.subsets]Z`. The hand-written popcount
table, the `x & (-x)` loop over set bits, the shift-and-xor that drops the top bit and the
separate `k == 0` product are now `wait.len`, `for b in wait`, `wait.shifted(1)` and the
`Slot.last` / `Slot.first` optionals. The DP's single state for k = 0 is the product.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and the package's 6 cases pass. On the max case the port takes 0.29 s, the v0.2 port
0.19 s and the accepted C++ 0.18 s (limit 0.5 s), all with the same output.
