# ioi12p4: Ideal City

## Dream

```cp
let at: IdealCity<Block> = read()  // connected, no holes, generated as such
fn crossings(blocks: List<(Int, Int)>) -> Int:
    let runs = blocks.sorted().runs(by: (a, b) => a == (b.0, b.1 - 1))
    let tree = Rooted(Graph.undirected(touching(runs)), runs.first!)
    sum(w * (n - w) for w in tree.subtree_sizes() except the root)
```

## "Connected, no holes" is not a type
- **Where:** the input type says nothing about the city's shape; gen.py builds
  one family of ideal cities (one run per column).
- **Severity:** low.

## A pair-keyed HashMap fails in g++
- **Where:** `run_of: Map<(Int, Int), Int>` with its comment; the HashMap form
  type-checks and then fails to compile in the generated C++.
- **Severity:** medium (Core).

## Unnamed pairs can't build a `Graph`
- **Where:** `undirected_of_pairs(edges)`: `Graph.undirected` wants
  `(u:, v:)` records and rejects `(u: Run, v: Run)` built in a comprehension
  ("takes a List<(Run, Run)>"), so the port uses the adjacency-list helper.
- **Severity:** low.

## Subtree sizes are a loop
- **Where:** `under` and the `tree.at.rev` loop (shared with ioi10p6's
  `Rooted.subtree_sums`).
- **Severity:** low.

## The judge's function task has no CP Lang form
- **Where:** `orig.cpp` wraps submission 4951048 calling `DistanceSum`.
- **Severity:** high.
