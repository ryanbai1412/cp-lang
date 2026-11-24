# noip19p6

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Cold review: the centroid, subtree sizes and heavy children come from Rooted; the four counts over subtree sizes are Fenwick<Size, Int> over a typed Size domain instead of the old hand-written segment tree. The recursive walk keeps the add/fold/undo pairs adjacent, so each Fenwick's invariant is local.

## Checklist failures

- `walk` is a recursive fn that mutates four global `Fenwick`s and a global `total`. The add/fold/undo pattern around `for k in t.children(x): walk(k)` is the C++'s DFS, unrolled by hand.
- `want = Size(max(n - 2 * t.size(x), 0))..=Size(n - 2 * heavy[x])` clamps a computed size into the domain by hand.
- `heavy` maps a missing heavy child to `0` (`?? 0`), and `second` does the same, as an R5 sentinel size.

## Gaps that need the language

- The recursive DFS over global counters, and clamping into `Size`: research/frictions/noip19p6.md.
