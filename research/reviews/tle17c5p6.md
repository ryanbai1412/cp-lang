# tle17c5p6

Score: 3/5
Dream: 4/5

Cold review: light(k) by memoised evaluation and a `last_true` for the flip (lines 18-37), the header's argument, but the memo is a global `[Wire]?Bool` reset by hand where a nested `@memo` already resets per call.

## Checklist failures

- R3: `value` with the reset loop in `light` (lines 18-28) is a hand memo; a `@memo fn` inside `light` is fresh on each call today.

## Owed rewrites

- Move `out` inside `light` as `@memo fn out(w: Wire) -> Bool`.

## Language gaps

- None. The friction's "`@memo` has no scope" is corrected in research/frictions/tle17c5p6.md.

## Dream notes

- `gates[w.label - n - 1]` indexes by hand where the port's `Gate(w.label - n)` is typed.
