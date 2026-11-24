# aac4p3

Score: 4/5
Dream: 5/5

Cold review of `port.cp` against `dream.cp` (the same code bar the header lines), without the C++.
Slopes are reduced to lowest terms with a positive denominator and deduplicated with `Set`. Each
direction then tallies the points by `b * y - a * x` and adds `c * (c - 1) / 2` per line, which is
the editorial.

## Checklist failures
None in the code. The answer is laid out over seven lines because `cpl fmt` breaks the `(a, b)`
pattern instead of the generator's `for` clauses (lines 22-29); that is in the friction.

## Owed rewrites
None.

## Language gaps
- `cpl fmt` breaking a short destructuring pattern in a long generator: in the friction (fmt).

## Dream notes
Reads like the editorial. `assert all(s.k != 0 and s.d != 0 ...)` is the visible check for a
nonzero bound a range type cannot state.
