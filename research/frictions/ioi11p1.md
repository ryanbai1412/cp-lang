# ioi11p1: Tropical Garden

## Dream

```cp
let walk = FunctionalGraph([State] s => next(s))
for by_best in Bool:
    let goal = state(F(p), by_best)
    let dist = walk.dist_to(goal)   // in-tree distances to goal
    let lap = walk.cycle_len(goal)  // ?Int
    ...
```

## A functional graph's reverse BFS and cycle length are hand-built
- **Where:** `back` (the reversed successor lists), `bfs`, and reading the
  goal's cycle length from `dist[next(goal)]`. These are the stock queries on
  a functional graph; a std `FunctionalGraph` would answer both.
- **Severity:** medium.

## A state that packs a fountain and a bit is index arithmetic
- **Where:** `State = 0..2 * n` with `state(f, by_best)` and `s.ord / 2`;
  a product domain `F x Bool` would carry both parts.
- **Severity:** low.

## The judge's function task has no CP Lang form
- **Where:** `orig.cpp` wraps submission 3732440 in a grader calling
  `count_routes` and `answer`.
- **Severity:** high.
