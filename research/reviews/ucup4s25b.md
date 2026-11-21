# ucup4s25b

Score: 4/5
Dream: 4/5

The header (lines 1-3) gives the plan, and `SparseTable` with `max` over `(merge_time, boundary)` is the right object. The simulation uses sentinels: a bottom ball of weight `inf` (line 14), a final ball of weight `2 * inf` (line 17), and `(-1, -1)` merge times (line 10). Direction is a `Bool`. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Sentinels (lines 10, 13-14, 17): `inf`, `2 * inf`, `(-1, -1)`.
- `dir: Bool` compared to `c[0] == 'R'` (lines 5, 21) instead of an enum read from the input.
- Raw `Int` positions wrapped in `B(...)` everywhere (lines 22, 27, 38-39); `l_raw - 1` (line 37).
- The loop over `0..=n` reads input only when `i < n` (lines 16-22): the sentinel ball is folded into the loop body.
- Unbounded `n`, `q` (lines 7, 34).

## Owed rewrites

- `enum Dir`; read balls into an array first and flush the stack after the loop; `?` merge times; typed 1-based reads; a dream.

## Language gaps

No friction report exists. None needed.

## Cold review (HEAD)

Dream 4/5 (new; port = dream), port → 4/5, rescored on current main. It checks with zero errors. The header derives the stack simulation, the one-merge-per-boundary fact and the query split.
- Minor: `n: Int` and `q: Int` (lines 11, 35) have no bounds. `Ball.bounds.nth(i.ord)` (line 21) converts a ball to the cut before it through `.ord`. The `(merged_at[b], b)` pair (line 33) carries `b` only to get an arg-max.
