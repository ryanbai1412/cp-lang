# `std.seq` matched benchmarks

Matches `std.seq` scan and rotation helpers against `std::partial_sum` and `std::rotate` in C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs asserted equal).

```sh
cargo build --release -p cpl-cli
bash bench/seq/run.sh
```

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
|---|---:|---:|---:|---:|---:|
| `scan` | `a.scan(+)`, `a.scan(min)`, `a.suffix_scan(max)` on `[Pos]Int` (`n = 10,000`, 2,000 rounds) | 122.7 ms | 126.4 ms | 0.97x | 5,506 B |
| `scan_in_place` | `v.scan_in_place(min)`, `v.suffix_scan_in_place(max)` on `List<Int>` (`n = 200,000`, 400 rounds) | 92.5 ms | 83.8 ms | 1.10x | 4,546 B |
| `rotate` | `v.rotate_left(k)`, `v.rotate_right(k / 3)` on `List<Int>` (`n = 200,000`, 200 rounds) | 50.2 ms | 48.9 ms | 1.03x | 8,900 B |
| `slice_sort` | `xs[l..r].sort()` vs `std::sort(xs.data() + l, xs.data() + r)` on `List<Int>` (`n = 200,000`, `len = 100,000`, 40 rounds) | 213.8 ms | 202.9 ms | 1.05x | 6,379 B |
| `slice_nth_element` | `xs[l..r].nth_element(k)` vs `std::nth_element(xs.begin() + l, xs.begin() + l + k, xs.begin() + r)` on `List<Int>` (`n = 200,000`, `len = 100,000`, 80 rounds) | 40.3 ms | 35.4 ms | 1.14x | 8,814 B |

Summary (5 matched benchmarks): **p50 = 1.05x** (target <= 1.10x), **max = 1.14x** (target <= 1.50x).
