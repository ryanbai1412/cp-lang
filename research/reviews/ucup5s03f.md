# ucup5s03f

Score: 2/5
Dream: none (no dream file)

Baseline review, cold, on main 4cf4b2b1. The port checks with zero errors. No dream exists, so the port is capped at 3. The header is only a title.

## Checklist failures

- No derivation: the two in-place difference passes per `k` (lines 15-16), the `ways[i] ** freq[i]` product (line 17) and the final `/ 2^n` (line 18) are unexplained.
- `a` is read as `[0..=m]`, i.e. `m + 1` values (line 6); say whether that matches the statement.

## Rewrites owed with today's language

- Write a header deriving the formula, name what `ways` holds after `k` passes, and write a dream.

## Gaps that need the language

- None found.
