# Friction report: dwite10c3p3

No gap. The width is `0..=30` as stated; the v0.2 port typed it `1..=30` and crashed on width 0, where the accepted C++ prints 1 (stress found it). Counts are `Mod<1_000_000>` and the subset loop filters with `if stands(...)`.

## Measurements

gen.py --max (widths 30, 29, 0, 1, 30): port 0.00 s, accepted C++ 0.00 s.
