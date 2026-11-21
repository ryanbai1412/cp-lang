# `std.dp` against matched C++

Each `NAME.cp` calls one DP entry point; `NAME_matched.cpp` is the
same algorithm written the way a contestant writes it in C++ (same tree
layout, same loop order, the cost inlined). `gen.py` writes the inputs.

```sh
bash bench/dp/run.sh
```

`run.sh` builds every pair, asserts equal outputs, and prints each ratio
and their p50 and max. `slope` is the `cf2084g2` port (`std.slope.Concave`)
at n = 5e5 against the original's lazy slope treap, rewritten from C++23
to C++17 in `slope_matched.cpp`.

Best of 5 wall-clock runs, g++ 11 `-O2`, equal outputs on every pair:

| bench | what | C++ | CP Lang | ratio | peak RSS C++ / CP |
|---|---|---|---|---|---|
| `frog3` | `LiChao<Line, Int>`, 2e5 lines and queries | 0.031 s | 0.028 s | 0.91x | 5.5 / 8.8 MB |
| `dnc` | 20 `dnc_layer`s, n = 2e5 | 0.194 s | 0.143 s | 0.74x | 16.0 / 14.8 MB |
| `knuth` | `knuth`, n = 3000 | 0.140 s | 0.174 s | 1.24x | 109 / 144 MB |
| `mono` | `std.window.MonoDeque` window min, n = 5e6 | 0.325 s | 0.329 s | 1.01x | 3.6 / 107 MB |
| `monge` | `monge_path`, n = 1e6, `(pre[i] - pre[j])^2 + c` | 0.085 s | 0.070 s | 0.82x | |
| `aliens` | `aliens` over an O(n) penalized DP, n = 1e6, k = 5000 | 0.132 s | 0.090 s | 0.68x | |
| `slope` | `std.slope.Concave`, `cf2084g2` at n = 5e5 | 0.099 s | 0.172 s | 1.74x | |

`run.sh` on the seven: p50 0.82x (target 1.1x), max 1.74x (target 1.5x).
The first four rows are the older measurements; `run.sh` gives frog3
0.79x, dnc 0.94x, knuth 1.23x, mono 0.40x.

`slope` is over the max. `perf` puts 60% of its time in `LazyTreap`'s
`put`, `ins` and `push_tag`: every tag push calls the stored `apply` and
`compose` closures and copies an optional tag, where the C++ applies the
pair in place, and the node list grows without a reserve.

`knuth` keeps its split table as 8-byte `Int`s where C++ uses `int`; it
reaches parity once 32-bit storage (`I32`) lands in the core. `mono`'s
memory is the whole input held as a string plus the array it is read
into; the C++ streams its input, so its memory is not matched. The C++
side reads with `scanf`, the CP side with its own buffered reader.
