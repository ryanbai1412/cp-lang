# joi16fp2

Score: 3/5
Dream: 3/5

Cold review: the greedy per stretch matches the header, but the walk over consecutive express stops is `E.at(e + 1)` by hand, and one loop counts the express stop and the further stops behind a `stop == 0` flag.

## Checklist failures

- R3: `for e in E: let Some(f) = E.at(e + 1) else: continue` (lines 37-39) walks adjacent stops by hand; std `express.adjacent` already gives the consecutive pairs on a table (`cpl run` of `for (x, y) in a.adjacent` over a `[E]Int` prints the pairs), so the friction's "would need a named `adjacent()`" claim does not hold.
- R2: `if stop == 0: reached += served else: gains.push(served)` (lines 45-46) mixes two quantities in one loop.
- R5: `let last = if fast * (stations - 1) <= limit: 1 else: 0` (line 48) counts the last station through a 0/1 `Int`.

## Owed rewrites

- `for (from, to) in express.adjacent` in place of the `E.at(e + 1)` walk.

## Language gaps

- `top(k)` for the largest `spare` gains is in research/frictions/joi16fp2.md; the `adjacent()` section there is corrected to say `.adjacent` exists.

## Dream notes

- `sum(first of each stretch)`, `served(from, at)` and `stops(from, to)` are prose, not code: the dream never writes the per-stretch walk, the core of the solution.
