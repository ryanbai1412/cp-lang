# ccc22s5

Score: 3/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the tree DP states `done`, `push`, `need`.
- R2: a tree DP over the influence tree: the editorial, with the states as three parallel `?Int` arrays (`push`, `done`, `need`).
- R4: `min(done[c], push[c])` is written four times (L29, 30, 35, 36); name it once per child.
- R5: `?Int` lifts by hand where `+=` does not (reported).

## Owed rewrites

- Name `best = min(done[c], push[c])` once per child.

## Language gaps

- `+=` does not lift to `?Int` (in the report).
- Three-way `min` nests (in the report).

## Dream notes

Reads like the editorial.
