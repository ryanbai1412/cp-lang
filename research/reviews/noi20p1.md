# noi20p1

Score: 3/5
Dream: 3/5

Cold review on main c8a46b201: port, dream and friction report, not the C++. The friction report's `## Dream` says the port is the dream, so both are judged on this file. The speed work (sentinel form kept between products) is measured and owned by std.

## Checklist failures

- Line 24 flattens (city, wait) into `State.nth(c.ord * 5 + wait)`, with the 5 also on line 18 (idiomatic-code/R4, R6).
- Lines 40-43 binary-expand the gap by hand (`(to - now) / 2 ** b % 2 == 1`) over a hand-built power list (lines 32-33) (idiomatic-code/R3).
- Line 50 prints `?? -1` for an unreachable home: fine, the statement's -1.

## Owed rewrites

- None.

## Language gaps

- A matrix power over a semiring applied to a row (`best * day ** gap`) that keeps the powers cached: the `## Dream` names std `MatPow` but no friction item routes it with a run.

## Dream notes

- "The port is the dream" keeps the C++ construction (hand bit-walk, flattened state index); the dream should read `domain State = (City, 0..5)` and `best = best * day ** (f.day - now)` (cap 3).
