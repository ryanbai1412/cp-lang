# atcoder_dp_s

Score: 4/5
Dream: 5/5

Cold review against the idiomatic checklist.

- R1: K's length bound is missing from `// Bounds:`; no `original:` line.
- R2: `below[r]` plus K's own residue `own` is the standard tight/non-tight digit DP.
- R5/R6: `Rem = 0..d` makes the residue a domain and `Rem.wrap` keeps it in range.
- R8: `Int(own != Rem.first)` subtracts K itself when its sum is not divisible; the
  trailing comment makes it legible, but `if own == Rem.first: 0 else: 1` would read
  closer to the statement.
- R10: 1.38x, reported with an owner (friction 1).

## Checklist failures

R1 (bound), R10 (reported).

## Owed rewrites

State K's length in the header.

## Language gaps

`Domain.wrap` and `Mod` add lowering (Core).
