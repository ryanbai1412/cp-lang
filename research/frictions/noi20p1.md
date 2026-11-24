# Friction Report: noi20p1

The port keeps one max-plus matrix per day over 5n (city, days still to travel) states, stored
as nested `[State]?Int` rows whose missing entries are `None` (the old `NONE = -1e18` sentinel
is gone). It squares to 2^b days and advances the best row between festivals. Every read is
typed by the statement's bounds.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and --max gives the same output.

## Remaining gap: speed

On --max the AC (3890258, avx2 and Ofast pragmas) takes 0.12-0.13 s and 21.1 MB. The port, on
the `noi` judge of its cp.toml (`isa = "avx2"`: the AC's pragma was accepted there), takes
0.28 s and 18.4 MB. `?Int max=` is one branch-free max, and the max-plus row loop is one
vectorised pass (`cpl::keep_max_plus`) whose adds fold their overflow into one flag. Without
that flag the same pass runs in 0.16-0.18 s. So even with no overflow checks at all, the `none`
select and the nested rows leave about 1.4x. The port stays out of v03 until a lowering closes
that gap without dropping the checks.

## Dream

The port is the dream: a closed max-plus semiring over `?Int` that the std `MatPow` could take,
once statically known ring ops compile to direct, vectorisable loops.
