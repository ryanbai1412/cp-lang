# Friction Report: noi20p1

The port keeps one max-plus matrix per day over 5n (city, days still to travel) states, stored
as nested `[State]?Int` rows whose missing entries are `None` (the old `NONE = -1e18` sentinel
is gone). It squares to 2^b days and advances the best row between festivals. Every read is
typed by the statement's bounds.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and --max gives the same output.

## Remaining gap: speed

On --max the port takes 0.47-0.53 s, and the AC (3890258, avx2/Ofast pragmas) 0.135 s. The
same program with an Int sentinel runs 0.22 s, so about half the gap is ?Int codegen: `c[l]
max= Some(via + w)` emits an overflow-checked add plus a `has_value` test on the Some it just
built. The rest is vectorisation. Both are with Core. The port stays out of v03 until they land.

## Dream

The port is the dream: a closed max-plus semiring over `?Int` that the std `MatPow` could take,
once statically known ring ops compile to direct, vectorisable loops.
