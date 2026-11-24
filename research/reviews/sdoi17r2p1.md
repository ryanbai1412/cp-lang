# sdoi17r2p1

Score: 3/5
Dream: pending (no dream.cp; the friction's `## Dream` is the target)

Cold review on main 377955a60, against the idiomatic checklist. Checks clean; agrees with the accepted C++ within 1e-4 at `--max` while using 3.5 MB against its 120 MB. The header (lines 3-9) explains the exact window DP and the CLT switch. `dice <= 800` is named once as `exact` (this commit). No dream.cp, so the port is capped at 3.

## Checklist failures

- `normal_below` (lines 15-25) is an 11-line erf series with hand cut-offs; in the friction report (stdlib).

## Rewrites owed with today's language

- None.

## Gaps that need the language

- `erf` / `normal_cdf` in std.
