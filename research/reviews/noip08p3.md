# noip08p3

Score: 3/5
Dream: none

Cold review: the two-paths-at-once DP over (step, row, row) the header derives, with the ordered-rows argument (lines 18-34). It has no dream, so the port is capped at 3.

## Checklist failures

- R6: rows and the derived column are `Int` (lines 24-31), with the bounds test `b2 < n and d + 1 - a2 < m` by hand; reported.

## Owed rewrites

- None.

## Language gaps

- Computed coordinates on a grid table (`kind.at(r, c)` returning an option): in research/frictions/noip08p3.md.

## Dream notes

- Neither dream.cp nor a `## Dream` section exists. The friction's ideal-code line is a fragment; the dream should write `best` over the two rows with `kind.at`.
