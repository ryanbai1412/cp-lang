# shoi16p5

Score: 3/5
Dream: pending (no dream.cp; the friction's `## Dream` is the target)

Cold review on main 377955a60, against the idiomatic checklist. Checks clean; agrees with the reference at max at 0.8x its time and memory. Each position is an `Affine`, a point update is one `set`, and the answer is the chain's constant: the editorial. No dream.cp, so the port is capped at 3.

## Checklist failures

- The opposite affine monoid is rebuilt from `Affine.id()` and a swapped `then`; in the friction report (stdlib, `Monoid.flip`).

## Rewrites owed with today's language

- None.

## Gaps that need the language

- `Monoid.flip()`.
