# noip00p4

Score: 3/5
Dream: 3/5

Cold review on main 6a989896d: port, dream and friction report, not the C++. The lockstep two-walker DP over anti-diagonals is the header's argument; optional states mark unreachable pairs. The dream is a sketch, so the score is provisional until it is rewritten.

## Checklist failures

- Line 16 types the grid `[Row, Row]`: columns reuse the row domain, and line 36 builds a column as `Row(k + 2 - Int(r))` (idiomatic-code/R6). Not in the friction.
- Lines 17-22 read until the `0 0 0` line with `while true` and `break`.
- Lines 29 and 33-36 derive the diagonal's rows and predecessors by index arithmetic: reported (friction 1).

## Owed rewrites

- Add `domain Col = 1..=n` and type the grid `[Row, Col]`.
- Rewrite the dream as code: `[Row, Row] (a, b) on diagonal k =>` and `grid.taken(a, b)` are undefined and it omits the start state.

## Language gaps

- Anti-diagonal cells and a max over optional predecessor pairs: friction 1.
