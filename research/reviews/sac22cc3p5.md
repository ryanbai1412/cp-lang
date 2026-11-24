# sac22cc3p5

Score: 3/5
Dream: pending (no dream.cp; the friction's `## Dream` is the target)

Cold review on main 377955a60, against the idiomatic checklist. The port checks with zero errors and agrees with the reference at `--max`. The header (lines 3-6) states the bound that makes `find_all` linear. No dream.cp, so the port is capped at 3.

## Checklist failures

- `letters` (line 13) converts every word and message into a `List<Letter>` because `AhoCorasick` takes only domain alphabets; in the friction report (stdlib).
- Memory is 3.2x the original's at max (17.8 MB against 5.6 MB); in the friction report (perf).

## Rewrites owed with today's language

- None beyond writing the dream.

## Gaps that need the language

- `AhoCorasick` over `Str`, or a `read<List<Letter>>()` that works.
