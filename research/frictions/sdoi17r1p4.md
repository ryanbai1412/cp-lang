# Friction report: sdoi17r1p4

`dream.cp` is now a copy of the port (added with this review): it checks, passes every package case and agrees with the accepted C++, as recorded by `corpus.py verify`. The ideal beyond today's language is below. The port is the ideal: Dinkelbach's method with exact integer ratios. Each round runs std's `max_weight_matching` under weights `den * a - num * b` and stops when the best matching weighs 0. `fixed(num, den, 6)` prints the exact ratio.

## Dream

The port as written.

There are no dream/port gaps. The accepted C++ binary-searches the ratio in floating point, with a Hungarian run per step (1.68 s at the max case against the 1.5 s limit). The port needs a few rounds and is exact.

## Measurements

- **Tokens:** port 160, accepted C++ 1263 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 1.68 s / 4 MB, with the same output.
- **Emitted C++:** 17775 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
