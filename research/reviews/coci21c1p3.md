# coci21c1p3

Score: 3/5
Dream: none

Cold review. There is no `dream.cp` and the friction has no `## Dream` (only `## Gap` and `## Measurements`), so the port caps at 3. The structure is the header's: the extra edge from a DSU (lines 18-22), the tree rooted at `a` without it (line 24), and a bottom-up DP over (vertex, colour, parent's colour) for each of the four guesses (lines 32-49).

## Checklist failures

- R7: `for e in E if not joined.unite(...): extra = e` (line 20) mutates the DSU inside the loop's filter.
- `plus` (lines 26-28) is optional addition by hand.

## Owed rewrites

- Add a `## Dream` to the friction.
- Unite in the loop body: `for e in E: if not joined.unite(edges[e].u, edges[e].v): extra = e`.
- The `## Gap` (`Graph.undirected` takes only a `[_]_` array) quotes no run; add the `cpl run` of `Graph.undirected([edges[e] for e in E if e != cut])` (R11).

## Language gaps

- `Graph.undirected` over a filtered edge list: in the friction.

## Dream notes

- None written.
