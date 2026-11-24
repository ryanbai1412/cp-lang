# cco16p3

Score: 3/5
Dream: 4/5

The header carries the editorial: blocks decide Flask, Moon and Sun by their edge and vertex counts, Talon is a degree-3 vertex, and Fox brute-forces cycles in small blocks. The match on `pattern` reads like that list. The block bookkeeping around it is hand-kept.

## Checklist failures

- idiomatic-code/R4: a block's vertex count is kept twice: `size` (lines 29-35, with a `counted: [B]?V` side array) and `members(b)` (lines 36-38), which rebuilds the same set from the edges.
- idiomatic-code/R3: `length` and `size` are hand counts the block structure should give.

## Owed rewrites

- Compute `members` once as `let members: [B]List<V>` and read `size` as `members[b].len`; `length` as a count over `E`. That drops `counted` and keeps one copy of each fact; it also replaces a scan of every edge per block with one pass.

## Language gaps

- No `Graph.has_edge` and no list-to-`Set` spelling (run quoted): research/frictions/cco16p3.md.
- `blocks(g)` partitions edges only: same report.

## Dream notes

- `bs.any(b => ...)` with `b.edges`, `b.vertices` and `b.vertices.cycles().any(exits)` reads like the header. It has no Source/Problem/Solution header and invents a `read<Graph.undirected(...)>` form; the `exits` closure walks `g.out(u)` twice.
