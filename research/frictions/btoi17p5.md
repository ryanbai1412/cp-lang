# Friction report: btoi17p5

Rethought. The v0.2 port read the cells one at a time into a `Fixed` struct. The v0.3 port reads `[0..k](sign: Char, row: 1..=n, col: 1..=m)` in one typed read. gen.py is new.

The accepted C++ subtracts one checkerboard whenever both families fit. That overcounts when both fit and no checkerboard does: `8 10 3 / - 2 1 / + 3 2 / - 7 8` has 32 + 128 - 0 = 160 grids, and the submission prints 159. orig.cpp counts the fitting checkerboards instead. The port and the v0.2 port agree on 160.

## An out-of-range `Int` key goes into a range-keyed `Map` silently
- **Where:** `alternating(1, [(0, ...) for f in cells])` (line 32) puts key `0` into
  `var seen = Map<1..=1e9, Bool>()` (line 20), because `pins` is `List<(Int, Bool)>`.
- **What felt wrong:** the key type says 1..=1e9 and nothing checks the conversion; the
  answer is right only because one consistent key is all the checkerboard needs.
- **Goal it hurts:** types carry the proofs; common mistakes hard to write.
- **Ideal code:** `alternating(1, [(1, ...) ...])`, with `0` a compile error or a trap.
- **Run:** `fn f(pins: List<(Int, Bool)>) -> Int:` / `var seen = Map<1..=1e9, Bool>()` /
  `for (line, b) in pins: seen[line] = b` / `seen.len`, then `print(f([(0, true)]))`, under
  `cpl run` (c8a46b201) prints `1`, with no error or trap.
- **General fix:** an `Int` used as a range-typed key is a checked conversion (a trap in
  checked builds, or a type error asking for `.at`/`!`), as for any other range-typed slot.
- **Severity:** medium (silent wrong-domain data).
- **Owner:** check.

## Measurements

gen.py --max (N = M = 1e9, K = 1e5): port 0.02 s, orig.cpp 0.03 s, same output.
