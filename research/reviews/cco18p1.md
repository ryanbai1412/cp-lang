# cco18p1

Score: 3/5
Dream: 4/5

The LCS-style DP from the header with rolling rows over `Game.cuts`: pair the two last games when they are rivals, or skip either. Clear, but the W/L strings are indexed by `.ord` while the scores are indexed by `Game`, and today's language already removes that.

## Checklist failures

- idiomatic-code/R6: `geese_won[g.ord]` and `hawks_won[h.ord]` (lines 17-18) cross `Game` through a raw `Int`.

## Owed rewrites

- Read the strings as `[Game]Char`: `let (w, s): ([Game]Char, [Game]1..=9) = read()` reads `WL 3 4` and indexes `w[Game(1)]` under 0.2, so the friction's first ideal already works. Indexing a `Str` by a domain does fail (`error[E0302]: expected Int, found G`).

## Language gaps

- `Str` not indexed by a domain: research/frictions/cco18p1.md; the `[Game]Char` read removes the need here.

## Dream notes

- The dream reads `[Game]Char` and keeps the full `[Game.cuts, Game.cuts]` table: the recurrence as stated. No header.
