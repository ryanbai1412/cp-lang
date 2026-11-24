# oly23practice5

Score: 3/5
Dream: 4/5

Cold review: the penalty counter over wrong-way runs, then the best penalty-free y per x (lines 16-48), as the header argues, but the base cost is written twice and the first-match is a `break` loop std already removes.

## Checklist failures

- R4: `n - even[x] - odd[y]` is spelled at lines 37 and 46; a `fn cost(x, y)` says it once.
- R3: lines 44-47 search for the first penalty-free `y` with `break`; `by_odd.first(y => y != x and (x, y) not in pen)` runs today.
- R1: the header does not say why a run of length L costs `(j - i) / 2` (line 24).

## Owed rewrites

- `fn cost`, and the per-x search as `by_odd.first(...)`.

## Language gaps

- The alternating-run scan by hand (lines 19-24): in research/frictions/oly23practice5.md. Its "`first(where:)`" gap is corrected there: `List.first(pred)` exists.

## Dream notes

- None.
