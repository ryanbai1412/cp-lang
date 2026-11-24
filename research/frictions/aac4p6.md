# aac4p6 friction

The port's code is `dream.cp`, so there is no dream/port gap. The port is over the 1.1x line.

## 1.21x slower than the accepted C++ at --max
- **Where:** the whole `port.cp`; N = 1e6 at `--max`.
- **What felt wrong:** port 0.030 s, accepted C++ 0.025 s (best of 10, same input, unlimited
  stack). `perf record` on the port: 57% in `prog::solve()` (the deque sweep) and 23% in
  `prog::show_1(std::vector<long long> const&, int)`, which prints the 1e6 answers.
- **Goal it hurts:** performance.
- **Ideal code:** the port as written (`print(a)`).
- **General fix:** printing a list of a million `Int`s should run as fast as the C++'s
  hand-rolled output loop. The print path is the first place to look; the deque sweep is a
  plain `Deque<Int>` with `pop_while`, `push_front` and `pop_front`.
- **Severity:** low (5 ms absolute).
- **Owner:** Core (codegen performance). Repro: `gen.py --max | port` against `orig.cpp`.

## Measurements
- Tokens: port 242, original 1,218.
- `--max` (best of 10): port 0.030 s / 11 MB, accepted C++ 0.025 s / 7 MB. Same output.
- Emitted C++: 19,978 bytes.
