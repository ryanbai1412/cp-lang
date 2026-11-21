# `std.seq` matched benchmarks

Matches `std.seq` scan and rotation helpers against `std::partial_sum` and `std::rotate` in C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs asserted equal).

```sh
cargo build --release -p cpl-cli
bash bench/seq/run.sh
```

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
|---|---:|---:|---:|---:|---:|
| `scan` | `a.scan(+)`, `a.scan(min)`, `a.suffix_scan(max)` on `[Pos]Int` (`n = 10,000`, 2,000 rounds) | 156.0 ms | 128.8 ms | 1.21x | 6,068 B |
| `scan_in_place` | `v.scan_in_place(min)`, `v.suffix_scan_in_place(max)` on `List<Int>` (`n = 200,000`, 400 rounds) | 87.5 ms | 84.5 ms | 1.04x | 4,562 B |
| `rotate` | `v.rotate_left(k)`, `v.rotate_right(k / 3)` on `List<Int>` (`n = 200,000`, 200 rounds) | 52.4 ms | 49.0 ms | 1.07x | 8,915 B |

Summary (3 matched benchmarks): **p50 = 1.07x** (target <= 1.10x), **max = 1.21x** (target <= 1.50x).
