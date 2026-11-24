# aac4p2

Score: 5/5
Dream: 5/5

Cold review of `port.cp` against `dream.cp`, without the C++. The header proves the monotone
predicate ("some light in 1..i is off") and derives `CAP`. Then `prefix_fold` over a capped `lcm`
monoid and one `I.first_true` per query give the editorial in four lines. Reads are typed from
the statement and `none: -1` prints the empty search.

## Checklist failures
None found. `on_until[i.r]` indexes `prefix_fold`'s `[D.cuts]T` result by the cut after `i`,
which is the std shape.

## Owed rewrites
None. The port streams the queries where the dream reads them all first; the friction measures
both (0.077 s / 5 MB against 0.073 s / 8 MB), so streaming is a choice, not a gap.

## Language gaps
None.

## Dream notes
Reads like the editorial.
