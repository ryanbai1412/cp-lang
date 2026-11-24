# dmopc21c2p6

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Offline: the passes, value asks and place asks are collected, then a parallel binary search finds when each position first holds a big value, with a `Fenwick` over positions.

## Checklist failures

- `value_asks` stores `at: Int` (`i - 1`) instead of a `Pos`, so the 1-based read is shifted by hand.
- `cut(c) = Pos.cuts.at(max(c, 0))!` clamps a computed cut and unwraps it.
- `is_big(i, k)` takes positions as `Int` and converts with `Pos(i - k)`, and tests `bigs.get(...) == 1` on a 0/1 count.
- The three query kinds are dispatched by `kind: 1..=3` with `if`/`elif`/`else`.

## Gaps that need the language

- Positions from 0: research/frictions/dmopc21c2p6.md.
