# ceoi22p1

Score: 4/5
Dream: 4/5

Cold review against `dream.cp`; the port is the dream with today's spellings (the friction lists them). The dream's header (lines 6-15) states the run invariant and why only the run crossing `n / 2` changes; the code keeps run lengths by leader in a `Fenwick<Card, Int>` (line 24), splits the crossing run (lines 47-57) and answers sorted queries with `first_ge` (lines 59-62).

## Checklist failures

- R5, minor: `var ans: [Q]Card = Card.first!` (line 42) fills with a placeholder card where the dream keeps `[Q]?Card = None`.
- Minor: `pos1 = qs[qi].i.ord + 1` and `pos1 - bit.prefix(leader.l) - 1` (lines 59-61) are 0/1-based index arithmetic by hand around the Fenwick.

## Owed rewrites

- Keep `ans: [Q]?Card` as the dream does, or fill `ans` by mapping the queries.

## Language gaps

- Next greater spelled `next_smaller(by: c => -c.ord)` (line 22): in the friction.

## Dream notes

- Reads like the editorial: the invariant is in the header, the structure is named. The same `+ 1` / `- 1` position arithmetic (lines 63-65) is its one translated spot.
