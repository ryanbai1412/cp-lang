# sac22cc4p4

Score: 4/5
Dream: 3/5

Cold review: count(T) - count(B - 1) with a two-pointer scan over max and min `Window`s (lines 16-31), the header's argument. Port score provisional until the dream is rewritten.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- One `Window` over (min, max) and a two-pointer helper: in research/frictions/sac22cc4p4.md, now with the `cpl run` errors.

## Dream notes

- `within(b - 1)` at b = 0 runs `keep` with limit -1, so the window empties and `w.all!` traps; the dream needs the guard or a `?` spread, and should read `b, t` with `t: b..=2e9` as the port does.
