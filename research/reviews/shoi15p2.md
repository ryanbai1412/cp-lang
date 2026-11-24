# shoi15p2

Score: 3/5
Dream: pending (no dream.cp; the friction's `## Dream` is the target)

Cold review on main 377955a60, against the idiomatic checklist. Checks clean; agrees with the reference at `--max` (1.2x its time, 0.8x its memory). The header (lines 3-9) states the run aggregate and how a transplant finds its end with `right_while`. `solid` (lines 17-18) is now a block body, so both branches read side by side. No dream.cp, so the port is capped at 3.

## Checklist failures

- The assign action (lines 29-34) is hand-built from a `?Bool` tag; in the friction report (stdlib).
- Line 49 mixes positions and cuts: `Cell.nth(min(short.ord, to_r.ord))` would read better as `min(short, to_r)` if `right_while` returned a `Cell`.

## Rewrites owed with today's language

- None.

## Gaps that need the language

- A stock `Action.assign`.
