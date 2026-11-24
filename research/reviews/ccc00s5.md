# ccc00s5

Score: 4/5
Dream: 3/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header derives
the half-line per rival, so each sheep's entry points form an interval, plus the
same-x-lower-y rule. The port computes the two interval ends as exact `Ratio`s. The only gap,
decimals read through floats and scaled, is in the friction.

## Checklist failures
- idiomatic-code/R5: `read_at: [n](x: F64, y: F64)` then `round(s.x * SCALE)` (lines 14-15);
  in the friction (no exact bounded decimal type).

## Owed rewrites
- The friction's 0.07 s against the C++'s 0.00 s gives no ratio. Time it with repetitions;
  file a perf friction if it is over 1.1x.

## Language gaps
- A bounded fixed-point decimal type: in the friction.

## Dream notes
`q.x^2` (four times) should be `q.x ** 2`, the canonical power. `max(cut(q) for q in sheep if
q.x < p.x)` is empty for the leftmost sheep, and `max(0, ...)` doesn't say what an empty `max`
is (caps at 3). Keep the port's `[Ratio(0, 1)] + [...]` bounds.
