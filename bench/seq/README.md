# `std.seq` matched benchmarks

Matches `std.seq` scan and rotation helpers against `std::partial_sum` and `std::rotate` in C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs asserted equal).

```sh
cargo build --release -p cpl-cli
bash bench/seq/run.sh
```

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
|---|---:|---:|---:|---:|---:|
| `scan` | `a.scan(+)`, `a.scan(min)`, `a.suffix_scan(max)` on `[Pos]Int` (`n = 10,000`, 2,000 rounds) | 125.2 ms | 127.8 ms | 0.98x | 5,498 B |
| `scan_in_place` | `v.scan_in_place(min)`, `v.suffix_scan_in_place(max)` on `List<Int>` (`n = 200,000`, 400 rounds) | 86.8 ms | 83.7 ms | 1.04x | 4,594 B |
| `rotate` | `v.rotate_left(k)`, `v.rotate_right(k / 3)` on `List<Int>` (`n = 200,000`, 200 rounds) | 50.0 ms | 49.6 ms | 1.01x | 8,955 B |

Summary (3 matched benchmarks): **p50 = 1.01x** (target <= 1.10x), **max = 1.04x** (target <= 1.50x).
