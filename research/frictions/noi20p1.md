# Friction Report: noi20p1

The port keeps one max-plus matrix per day over 5n (city, days still to travel) states, stored
as nested `[State]?Int` rows whose missing entries are `None` (the old `NONE = -1e18` sentinel
is gone). It squares to 2^b days and advances the best row between festivals. Every read is
typed by the statement's bounds.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and --max gives the same output.

## Remaining gap: speed

On --max the port took 0.47-0.53 s, and the AC (3890258, avx2/Ofast pragmas) 0.135-0.15 s.
`?Int max=` now lowers to one `std::max<OptInt>` on the niche's raw value (no `has_value` test
on the Some it just built, no branch), which brings the port to 0.33-0.36 s at the same 19.8 MB.
The rest of the gap is the AC's `#pragma GCC target("avx2")` and `optimize("Ofast")`. Built at
plain -O2 without those two lines, the AC takes 0.33 s, the same as the port. With -mavx2 alone it
takes 0.18 s. A hand-made branch-free copy of the port's loops (a select on the niche, a local
bound, no overflow check) still doesn't vectorise and runs in 0.30 s. So no lowering reaches 1.1x
at the judge profile's `isa = "x86-64"`; that needs the profile's planned `isa = "avx2"` target
pragmas (spec/tools/judge-profiles.md) or a pragma-free reference. The port stays out of v03 until
one of those is decided.

## Dream

The port is the dream: a closed max-plus semiring over `?Int` that the std `MatPow` could take,
once statically known ring ops compile to direct, vectorisable loops.
