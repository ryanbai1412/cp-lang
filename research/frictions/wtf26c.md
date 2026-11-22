# wtf26c friction

## No 2D convolution or binomial inversion

- **Goal it hurts:** express
- **Severity:** medium
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/wtf26c.cp:30,38`
  (`convolve_2d(claim, dp, shape: (rows, cols))`, `labelled.binomial_inverse()`)
- **Actual:** `proto/ports/ecnerwala/wtf26c.cp:22-27`
  flattens with row stride `2 * rows - 1` around one `convolve`; the first
  inversion is a double loop (`wtf26c.cp:48`).
- **General fix:** `std.ntt.convolve_2d(a, b, shape:)` over 2D arrays, and
  binomial-transform/inverse helpers (the `std.poly` EGF product with `e^{-x}`).

## Measurement

`n` stays `Int` (bound not recorded). n = 120 random sorted `a`, `g++ -O2`:
8.15 s vs 7.95-8.15 s, same output. Emitted C++ 28,174 -> 29,022 bytes.
