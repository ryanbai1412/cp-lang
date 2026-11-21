# wtf26d

Score: 1/5

A line-by-line translation: the stack of `Cond` constraints, `found_move`/`broke` flags and `-1`/`n` sentinels can't be followed without the C++. The header names steps but not the invariant that makes the greedy correct. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Flag loops (lines 27, 47, 50, 56-63): `found_move` and `broke` drive control flow.
- Sentinels (lines 22, 24, 26, 59: `n`, `-1`, `Cond(-1, n)`, `q[i + 1] = n`).
- Raw `Int` index spaces and 0-based shifts (lines 8-9: `read() - 1`); positions and values share `Int`.
- The same pop loop is written three times (lines 40, 45, 49).

## Owed rewrites

- Declare `domain Pos`/`Val` and read the permutations as `[Pos]Val`; use `p.inverse()`; hoist the triple `while pos < conds.last!.max_p: conds.pop()` out of the branches; return an enum instead of the `broke`/`found_move` flags.

## Language gaps

No friction report exists. Missing: whatever stops the lane writing the invariant stack as a named structure.
