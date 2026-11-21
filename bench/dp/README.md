# `std.dp` against matched C++

Each `NAME.cp` calls one DP entry point; `NAME_matched.cpp` is the
same algorithm written the way a contestant writes it in C++ (same tree
layout, same loop order, the cost inlined). `gen.py` writes the inputs.

```sh
cargo build --release -p cpl-cli
python3 bench/dp/gen.py /tmp
for p in frog3 dnc knuth mono; do
  target/release/cpl build bench/dp/$p.cp -o /tmp/$p.cpp
  g++ -std=c++17 -O2 -o /tmp/${p}_cp /tmp/$p.cpp
  g++ -std=c++17 -O2 -o /tmp/${p}_c bench/dp/${p}_matched.cpp
done
```

Best of 5 wall-clock runs, g++ 11 `-O2`, equal outputs on every pair:

| bench | what | C++ | CP Lang | ratio | peak RSS C++ / CP |
|---|---|---|---|---|---|
| `frog3` | `LiChao<Line, Int>`, 2e5 lines and queries | 0.031 s | 0.028 s | 0.91x | 5.5 / 8.8 MB |
| `dnc` | 20 `dnc_layer`s, n = 2e5 | 0.194 s | 0.143 s | 0.74x | 16.0 / 14.8 MB |
| `knuth` | `knuth`, n = 3000 | 0.140 s | 0.174 s | 1.24x | 109 / 144 MB |
| `mono` | `std.window.MonoDeque` window min, n = 5e6 | 0.325 s | 0.329 s | 1.01x | 3.6 / 107 MB |

`knuth` keeps its split table as 8-byte `Int`s where C++ uses `int`; it
reaches parity once 32-bit storage (`I32`) lands in the core. `mono`'s
memory is the whole input held as a string plus the array it is read
into; the C++ streams its input, so its memory is not matched. The C++
side reads with `scanf`, the CP side with its own buffered reader.
