# joi16op3

Score: 4/5
Dream: 3/5

Cold review: the connected-components insertion DP with the open-sides cost trick, each transition commented with what it does (lines 30-41). Port score provisional until the dream is rewritten.

## Checklist failures

- R1: `if n == 1: Z.one` (line 42) is a special case the header does not derive.

## Owed rewrites

- Derive the n = 1 case in the header, or drop it if `ways[1, c, 2]` already counts it.

## Language gaps

- The distinct-heights assert, `level` as a carried `var`, and the untyped `[0..=n, 0..=limit, 0..=2]` state: all in research/frictions/joi16op3.md.

## Dream notes

- `insert(ways, h - level)` hides all five transitions, the whole DP; the dream should write them (as the port does) and keep only the fold and the typed state as the improvement.
