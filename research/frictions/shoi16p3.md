# Friction report: shoi16p3

The port is now the body of `dream.cp` (verified), under the port's full header. The previous port matched the dream with small drift:
- `tables(c, above, below)` took the ranks the caller had already computed;
- `ys` was built with `(0..=n + 1).map(...).to_list()`;
- the start row was `if j.label == n - 1: 1 else: 0`.

## Dream

`dream.cp`, which the port now matches. There is no dream/port gap.

## Measurements

- **Tokens:** port 281, accepted C++ 1581 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.06 s / 3 MB, accepted C++ 0.01 s / 3 MB, with the same output.
- **Emitted C++:** 24412 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
