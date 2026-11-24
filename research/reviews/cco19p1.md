# cco19p1

Score: 4/5
Dream: 4/5

Every reachable board is searched, memoised on the board per side to move, and a position's chance is the mean over the mover's best `slack` moves. Reads like the header; the per-side map is a measured choice (1.4x with a tuple key) and is reported.

## Checklist failures

- idiomatic-code/R13: `type Cell = (Row, Col)` (line 17) is never used.

## Owed rewrites

- Drop `Cell`. The two `continue` guards (lines 34, 36) read better as `if` filters on the loops, as the dream has them.

## Language gaps

- The tuple key copies the board per lookup, so the memo is one map per side: research/frictions/cco19p1.md.

## Dream notes

- `@memo` over `(board, who)`, a comprehension of moves and `mean() ?? 0.0`: the editorial in six lines. `board.moved(p, q)` and `who.other` are proposals the friction does not list.
