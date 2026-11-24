# saspeed — Suffix Array speed test

## std `suffix_array` is 2.5x slower than the original's SA-IS
- **Where:** `port.cp`, `suffix_array(s)`.
- **What felt wrong:** at 10^6 letters the std call alone takes about 0.27 s native against
  0.11 s for the whole original (most of the gap is system time, so allocation).
- **General fix:** reuse buffers across SA-IS recursion levels in `std.strings.suffix_array`.
- **Goal it hurts:** fast.
- **Severity:** medium.

## Dream

The port.
