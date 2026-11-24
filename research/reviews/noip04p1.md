# noip04p1

Score: 3/5
Dream: 4/5

Cold review on main 6a989896d: port, dream and friction report, not the C++. The simulation is the header's. The friction's gap is not a gap: a top-level `return print(...)` inside the loop runs today (`for m in 0..3: if m == 1: return print(-m)` prints `-1`), so the dream's spelling is available and the `short` flag is avoidable.

## Checklist failures

- Lines 15-23 carry the first short month in `var short: ?Month` and branch after the loop, where `if cash < 0: return print(-Int(month))` works today (idiomatic-code/R7).

## Owed rewrites

- Return from the loop as the dream does; the friction now records that it runs.

## Language gaps

- None.
