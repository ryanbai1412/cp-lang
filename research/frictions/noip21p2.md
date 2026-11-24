# Friction report: noip21p2

There is no dream.cp. The port is the ideal: bits are added in order, with `dp[used, carry, ones]` over a `type Layer = [0..=n, 0..=n / 2, 0..=k]Z`. Taking `take` more positions multiplies by `C(n - used, take) w^take`. The carry bound `used / 2` is stated in the type's middle extent.

## Dream

The port as written.

There are no dream/port gaps. `var power = Z(1)` with `power *= w` is the running `w ** take`, kept by hand to avoid a log factor in the innermost loop. That is the editorial's choice, not a language gap.

## Measurements

- **Tokens:** port 250, accepted C++ 670 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.02 s / 3 MB, accepted C++ 0.18 s / 15 MB, with the same output.
- **Emitted C++:** 21781 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
