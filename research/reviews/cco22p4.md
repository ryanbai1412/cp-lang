# cco22p4

Score: 4/5
Dream: 4/5

Cold review against the port and the friction's `## Dream`. Children are `?Node` via `Node.at` (lines 21-22), counting is `Comb<Z>` (line 17), `spare` is an additive memoized fn (line 28) and `fill` is memoized over `(Node, Arrive)` (lines 33-54): the header's recurrence, top to bottom.

## Checklist failures

- None beyond the minor note below.
- R5, minor: `here: [Node]Int` (line 18) is filled by a counting loop (line 19) and `is_right` by another (lines 23-25); both are facts of the input, not state.

## Owed rewrites

- The memo friction quotes no run; add one. Note the dream as written still needs the guard before `Arrive(k)` is formed, since `a + h` can exceed `n`; the general fix is a memo that accepts an out-of-domain key with a guard, which the friction should say.

## Language gaps

- Guarded `@memo(array)` over a domain key (line 31): in the friction, without a repro.

## Dream notes

- The dream is a fragment (one fn replacing lines 31-34), judged as the port with that change. Read that way it states the recurrence; the header (lines 7-11) derives the stop-at-`u` term behind line 44's `- Int(...)`.
