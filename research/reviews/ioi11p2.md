# ioi11p2 review

Score: 4/5
Dream: 4/5 (the friction's `## Dream`: the gap's ideal over the port)

Scored cold against the checklist and the port.

Cold review: already at bar. Std `Rooted` over a weighted `Graph.undirected`, children before
parents via `tree.at.rev` (no recursion), small-to-large `HashMap` merging with `swap`, the
answer as `?Int` printed with `?? -1`, and roads read typed as `[Road](u: City, v: City, len:
0..=1e6)` with statement bounds. Nothing has a simpler std spelling; no change beyond the review.

## Checklist failures

- `var there: HashMap<...> = []; swap(mut there, mut reach[child])` takes a child's map by swapping in an empty one. A `take` (move out, leave empty) would say it once.
- `random_tree` sits in the port only as `read(gen:)`'s generator. It is test scaffolding that the solution reads past.

## Gaps that need the language

- Moving a value out of an array slot: research/frictions/ioi11p2.md.
