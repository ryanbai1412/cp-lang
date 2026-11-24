# Friction Report: noi20p1

The port keeps one max-plus matrix per day over 5n (city, days still to travel) states, stored
as nested `[State]?Int` rows whose missing entries are `None` (the old `NONE = -1e18` sentinel
is gone). It squares to 2^b days and advances the best row between festivals. Every read is
typed by the statement's bounds.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and --max gives the same output.

## Remaining gap: speed

The port is now a flat `Mat<?Int>` over `Semiring(max, +)`: `*` for the 2^b powers and a 1 x |State|
`Mat` for the best-joy row. std's `TropicalProduct for Int` runs the product: while every entry is
within 2^60 of 0 it holds `None` as -2^62 (std's own sentinel, never seen by the port) and runs an
unchecked, branch-free ikj pass; past the bound it leaves the product to the `?Int` loop, which traps
as written (std-matrix/max-plus-past-the-bound-traps-as-the-loop.cp).

Measured on --max, five runs each, noi judge (`isa = "avx2"`):

| Build | Time | RSS |
|---|---|---|
| AC as written (Ofast, unroll-loops, avx2) | 0.12-0.15 s | 21.1 MB |
| AC with O3 in place of Ofast | 0.12 s | |
| Port | 0.22-0.25 s | 20.3 MB |
| Port with Ofast in place of O3 | 0.24-0.28 s | |

So Ofast's reassociation is not the gap. perf on the port: 50% is the ikj inner max loop, 22% the
`?Int` -> flat copies of both operands (and the bound reduction), 12% filling the output, and the
rest the flat -> `?Int` copy back. The AC keeps its matrices flat between products; the port pays
two conversions per product. bench/math/maxplus (200x200 `**` 10^6) is 1.12x its C++.

## Dream

The port is the dream: a closed max-plus semiring over `?Int` that the std `MatPow` could take,
once statically known ring ops compile to direct, vectorisable loops.
