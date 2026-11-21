# `std.seq` matched benchmarks

Matches `std.seq` scan and rotation helpers against `std::partial_sum` and `std::rotate` in C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs asserted equal).

```sh
cargo build --release -p cpl-cli
bash bench/seq/run.sh
```

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
|---|---:|---:|---:|---:|---:|
| `scan` | `a.scan(+)`, `a.scan(min)`, `a.suffix_scan(max)` on `[Pos]Int` (`n = 10,000`, 2,000 rounds) | 124.0 ms | 127.5 ms | 0.97x | 5,512 B |
| `scan_in_place` | `v.scan_in_place(min)`, `v.suffix_scan_in_place(max)` on `List<Int>` (`n = 200,000`, 400 rounds) | 94.6 ms | 84.8 ms | 1.12x | 4,608 B |
| `rotate` | `v.rotate_left(k)`, `v.rotate_right(k / 3)` on `List<Int>` (`n = 200,000`, 200 rounds) | 51.7 ms | 48.4 ms | 1.07x | 8,962 B |
| `slice_sort` | `xs[l..r].sort()` vs `std::sort(xs.data() + l, xs.data() + r)` on `List<Int>` (`n = 200,000`, `len = 100,000`, 40 rounds) | 213.4 ms | 202.9 ms | 1.05x | 6,441 B |

Summary (4 matched benchmarks): **p50 = 1.06x** (target <= 1.10x), **max = 1.12x** (target <= 1.50x).
