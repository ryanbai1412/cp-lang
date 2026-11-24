# neerc10k

Score: 4/5
Dream: 4/5

Cold review: the least odd k, a BFS from a vertex of degree below k, then a greedy colouring in reverse BFS order with `find` over the free colours (lines 16-23), all derived in the header.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- `Paths` has no visit order (std `Paths` exposes `get`, `iter`, `reached`, `path` and `arcs`), so the port sorts by distance, O(n log n): in research/frictions/neerc10k.md.

## Dream notes

- None.
