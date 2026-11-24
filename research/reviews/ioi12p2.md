# ioi12p2 review

Score: pending
Dream: pending

Cold review: rethought. The old port hand-wrote a union-find over a flat `[0..5n]Int` (five layers
of n packed by `layer * n + ring`, -1 sentinels, raw Int rings) with an untyped candidate list.
Now rings are a `Ring` domain, the whole graph is a std `Dsu<Ring>` and each candidate k gets its
own `without[k]: Dsu<Ring>` and `deg_without[k, ring]`, with `Cand = 0..4` naming the hub and its
three neighbours. The hub is a `?Ring` (None until some ring gets three links) and `cand(h, k)`
derives the other candidates from its adjacency, which no longer changes. Links are `(Ring, Ring)` pairs, and both orders of an edge come from one loop. Reads are typed
(`-1..=n - 1`, where -1 -1 is CountCritical).
