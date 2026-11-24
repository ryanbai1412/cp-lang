# cco16p4

Score: 3/5
Dream: 3/5

The header states the invariant (row and column parities are all a 2x2 flip keeps) and the program keys each grid by them and counts equal pairs: O(G n^2), no simulation.

## Checklist failures

- idiomatic-code/R4: the Counter's count bound `0..=10_000` (line 17) restates the bound of `g` (line 11); `0..=g` says it once.

## Owed rewrites

- Spell the count bound as `0..=g`. The running `pairs += seen[key]; seen[key] += 1` waits on a Counter built from a sequence.

## Language gaps

- Counting equal pairs takes a running Counter: research/frictions/cco16p4.md.
- The key is two `List<Int>` because domain arrays are not hashable keys: same report.

## Dream notes

- `dream.cp` keeps a greedy builder where the port's header names the rule: it clears the top-left block cell by cell with `R(r + dr)` index arithmetic (caps at 3). The rewrite is the port's own idea: `Counter([(Row.map(r => parity(row r)), Col.map(...)) for grid in grids]).pairs`.
- Its header names another problem ("Colourful Squares-ish") and has no Source line.
- A dream under 4 is sent back and caps the port at 3 (research/reviews/README.md); the port itself reads at 4.
