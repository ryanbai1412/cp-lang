# Prefix / Diff / FillPrefix benchmarks

Each `X.cp`, compiled with `cpl build`, against matched hand-written C++17
(`X_hand.cpp`) using the same data layout (`int64_t` for `Int`, `uint32_t` for
`Mod<998244353>`) and algorithm, both compiled with `g++ -std=c++17 -O2`, best
of 5 runs on identical inputs.

```bash
cargo build --release -p cpl-cli
python3 bench/prefix/run.py
```

| bench | input | cpl (s) | hand C++ (s) | ratio | emitted (B) |
|---|---|---:|---:|---:|---:|
| prefix_1d | n=524k, q=10M | 0.048 | 0.038 | 1.25x | 8,241 |
| prefix_2d | 1024x1024, q=10M | 0.074 | 0.071 | 1.04x | 10,514 |
| diff_1d | n=524k, q=10M | 0.056 | 0.049 | 1.15x | 8,371 |
| diff_2d | 1024x1024, q=10M | 0.093 | 0.088 | 1.05x | 11,091 |
| fill_1d_fwd | n=10M | 0.057 | 0.066 | 0.88x | 11,404 |
| fill_1d_rev | n=10M | 0.062 | 0.068 | 0.91x | 11,551 |
| fill_2d_fwd | 2048x2048 | 0.045 | 0.034 | 1.33x | 14,266 |
| fill_2d_rev | 2048x2048 | 0.048 | 0.035 | 1.35x | 14,560 |

Summary: **p50 = 1.10x** (target `<= 1.10x`), **max = 1.35x** (target `<= 1.50x`).

Profiling findings >1.10x are logged in `research/frictions/prefix_bench.md`.
