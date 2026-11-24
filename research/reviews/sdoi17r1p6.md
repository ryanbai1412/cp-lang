# sdoi17r1p6

Score: 3/5
Dream: pending (no dream.cp; the friction's `## Dream` is the target)

Cold review on main 377955a60, against the idiomatic checklist. Checks clean; agrees with the reference at `--max` in 1.0x its time and 0.56x its memory. The header (lines 3-10) derives the slope from four sums and says why they are I128. No dream.cp, so the port is capped at 3.

## Checklist failures

- `Sums.first` (line 13) is not a sum: `join` takes its `min` (line 25) and the identity needs the sentinel `n + 1` (line 63). It exists because `Action`'s apply sees only the span length; in the friction report (stdlib).
- The field-wise `join` (lines 20-26) is spelled out; in the friction report (check).

## Rewrites owed with today's language

- None.

## Gaps that need the language

- `Action` apply over the segment's range; derived field-wise `+`.
