# Friction report: btoi17p5

Rethought. The v0.2 port read the cells one at a time into a `Fixed` struct. The v0.3 port reads `[0..k](sign: Char, row: 1..=n, col: 1..=m)` in one typed read. gen.py is new.

The accepted C++ subtracts one checkerboard whenever both families fit. That overcounts when both fit and no checkerboard does: `8 10 3 / - 2 1 / + 3 2 / - 7 8` has 32 + 128 - 0 = 160 grids, and the submission prints 159. orig.cpp counts the fitting checkerboards instead. The port and the v0.2 port agree on 160.

No language gap.

## Measurements

gen.py --max (N = M = 1e9, K = 1e5): port 0.02 s, orig.cpp 0.03 s, same output.
