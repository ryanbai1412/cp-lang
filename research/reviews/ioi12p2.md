# ioi12p2 review

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Cold review: rethought. The old port hand-wrote a union-find over a flat `[0..5n]Int` (five layers
of n packed by `layer * n + ring`, -1 sentinels, raw Int rings) with an untyped candidate list.
Now rings are a `Ring` domain, the whole graph is a std `Dsu<Ring>` and each candidate k gets its
own `without[k]: Dsu<Ring>` and `deg_without[k, ring]`, with `Cand = 0..4` naming the hub and its
three neighbours. The hub is a `?Ring` (None until some ring gets three links) and `cand(h, k)`
derives the other candidates from its adjacency, which no longer changes. Links are `(Ring, Ring)` pairs, and both orders of an edge come from one loop. Reads are typed
(`-1..=n - 1`, where -1 -1 is CountCritical).

## Checklist failures

- Each candidate's state is three parallel arrays (`without: [Cand]Dsu`, `deg_without: [Cand, Ring]Int`, `chains: [Cand]Bool`). A `struct` per candidate holding its Dsu, degrees and flag would keep them in step.
- Five top-level `var`s (`cycles`, `cycle_len`, `hub`, `links`, `deg`) are mutated from `link`, so the program's state is spread over globals.
- `cand(h, k)` indexes `adj[h][i.ord]`, an ordinal into the hub's adjacency list, to name a neighbour candidate.
- The two stages (before and after a hub appears) live in one `link` with an early `return`. Two named phases would read better.

## Gaps that need the language

- The grader interface turned into a stdin encoding: research/frictions/ioi12p2.md.
