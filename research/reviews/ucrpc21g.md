# ucrpc21g review

Score: pending
Dream: pending

Cold review: rethought. The old port kept every reachable (cell, held) state with a stamp table
(sentinel -1) and an event table where -1 meant a boulder. Both drops are monotone in the
popcorn held, so the most held per cell dominates: `held: [Row, Col]?Int` (None while
unreachable) and each second is two table comprehensions, the best of `p.adj4 + [p]` via
`present().max()`, then that second's drops. Popcorn counts and boulders are separate typed
tables, so no sentinel is left, and the bound drops from O(100 n^2 p) to O(100 n^2).
