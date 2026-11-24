# cco17p2

Score: 3/5
Dream: 3/5

The strip-peeling recurrence from the header, memoised on `(n, m)`, with an impossible split as `None` and the answer the join of both orientations. Compact and close to the editorial; the optional (fewest, most) join is the one hand-written piece, and it is reported.

## Checklist failures

- idiomatic-code/R1: the loop start `max(1, (m - 2 * n) / jump - 1)` (line 28) has a `- 1` of slack the header ("the rest must be <= 2n") does not derive.

## Owed rewrites

- Derive the lower end of `k` in the header, or say it is a safe under-estimate.

## Language gaps

- Joining optional `(fewest, most)` pairs by hand (`fn join`): research/frictions/cco17p2.md (a `MinMax` monoid).

## Dream notes

- `districts(m - k * jump, n)` is written twice in one expression (idiomatic-code/R4, caps at 3); bind the rest once per `k`, as the port does with `if let rest = ...`.
- `fold(MinMax.empty, for k in ...: ...)` is not a spelling the language has; a generator `fold`/`sum` over `k` says it.
- A dream under 4 is sent back and caps the port at 3 (research/reviews/README.md); the port itself reads at 4.
