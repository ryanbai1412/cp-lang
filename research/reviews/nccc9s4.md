# nccc9s4

Score: 3/5
Dream: 4/5

Cold review: rollback Mo's is the right algorithm and the header explains its parts, but it is hand-written with `Int` offsets, a filler answer table, and the mode packed into one `Int`; the friction still describes an optional-tuple version.

## Checklist failures

- R5: `best` packs (count, score) as `cnt[v] * n + v.ord` (line 25) and unpacks it with `V.nth(best % n)` (lines 34 and 51), a hand encoding where tuples already compare; the friction describes `best ?? (0, v)` instead.
- R5: `var ans: [Q]V = V.first!` (line 27) is a filler.
- R6: `add(j: Int, ...)` and the `qs[qi].l - 1` offsets (lines 30, 46) index scores by `Int`, not by a position domain.
- R4: the short-query and long-query paths both add, read and roll back the same counts (lines 32-35 and 49-52).

## Owed rewrites

- A `(count, value)` tuple best, unless the packed form is measured faster; then the numbers go in the friction.

## Language gaps

- No std rollback Mo's: in research/frictions/nccc9s4.md. The packed-`Int` best that replaced the optional tuple is filed there.

## Dream notes

- None.
