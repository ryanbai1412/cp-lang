# coi06p2

Score: 4/5
Dream: 3/5

The port answers road- and town-deletion connectivity queries with DFS low-links, bridge tests and stranded child subtrees. `let (tree, tree_road) = map.dfs_tree(Town.first!)` (L18) names the traversal and `fn stranded(c: Town, x: Town) -> ?Town:` (L26) names the vertex-cut invariant; the only loop left is the low-link fold, which the friction reports.

## Checklist failures
None unreported. The low-link loop (L19–23) is the friction's gap.

## Owed rewrites
- Rewrite the friction's dream line so it parses and checks against `tree.up`'s real shape; the R11 runs added to the friction show both problems.

## Language gaps
- A `min` fold over preorder positions for `tree.up` (in the friction). The run added to the friction shows the dream line as written does not parse (E0103 on infix `min`), and with `min(..)` spelled out `tree.up` on a DFS tree asks for `op`, `lift` and `finish` with a three-argument lift (E0303), so the gap is a signature mismatch to resolve, not yet a missing monoid.

## Dream notes
The dream names the right fold, but `tree.pos(v) min map.out(v)...` is not CP Lang syntax and it hides the empty-neighbour case behind `.min()`; 3 until it is a line that checks.
