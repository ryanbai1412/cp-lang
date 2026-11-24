# Friction Report: noi20p1

The port keeps one max-plus matrix per day over 5n (city, days still to travel) states, stored
as nested `[State]?Int` rows whose missing entries are `None` (the old `NONE = -1e18` sentinel
is gone). It squares to 2^b days and advances the best row between festivals. Every read is
typed by the statement's bounds.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and --max gives the same output.

## Speed

The port is a flat `Mat<?Int>` over `Semiring(max, +)`: `*` for the 2^b powers and a 1 x |State|
`Mat` for the best-joy row. std's `TropicalProduct for Int` runs the product: while every entry is
under 2^60 in size it holds `None` as -2^62 (std's own sentinel, never seen by the port) and runs an
unchecked, branch-free ikj pass; past the bound it leaves the product to the `?Int` loop, which traps
as written (std-matrix/max-plus-past-the-bound-traps-as-the-loop.cp). A product's result stays in
that sentinel form, with its largest `|entry|`, until an entry is read or set, so a chain of
products (the powers, then each best-row step) converts each matrix once, not twice per product.

Measured on --max, noi judge (`isa = "avx2"`), g++ 14 -O2:

| Build | Time | RSS |
|---|---|---|
| AC as written (Ofast, unroll-loops, avx2) | 0.12-0.15 s | 21.1 MB |
| Port, converting on every product | 0.22-0.25 s | 20.3 MB |
| Port, sentinel form kept between products | 0.12-0.13 s | 21.4 MB |

With conversions on every product, perf put 50% in the inner max loop, 22% in the `?Int` -> flat
copies of both operands, 12% in filling the output and the rest in the copy back; each 1 x 250
best-row step converted the whole 250 x 250 power. Ofast's reassociation was never the gap (the AC
with O3 in its place also runs in 0.12 s). bench/math/maxplus (200x200 `**` 10^6) is 1.02-1.07x
its C++.

`corpus.py verify --only max` builds the port for `codeforces`, not the package's `noi` judge, so it
times the port without the avx2 target the AC's own pragmas give it: 0.38-0.54 s there.

## Dream

The port is the dream: a closed max-plus semiring over `?Int` that the std `MatPow` could take,
once statically known ring ops compile to direct, vectorisable loops.
