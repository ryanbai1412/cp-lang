# aac4p2 friction

The port and `dream.cp` differ in one line, and it is a choice, not a missing feature: the
dream reads every query first (`let queries: [q]0..=1e9 = read()` then `for t in queries:`),
while the port reads each `t` inside the loop (`for _ in 0..q: let t: 0..=1e9 = read()`).
Both forms compile and give the same output. At `--max` (q = 1e6) the dream takes 0.073 s and
8 MB, the port 0.077 s and 5 MB, so streaming costs nothing and saves the 8 MB query array.
There are no gaps, so nothing is routed.

## Measurements

- Judge limits: 1000 ms, 256 MB, as the port's `limits:` line says; checked against the DMOJ API (`/api/v2/problem/aac4p2`), https://dmoj.ca/problem/aac4p2.
- Tokens: port 137, original 373.
- `--max` (best of 10): port 0.081 s / 6 MB, accepted C++ 0.102 s / 5 MB. Same output.
- Emitted C++: 15,204 bytes.
