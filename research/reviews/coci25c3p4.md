# coci25c3p4

Score: 3/5
Dream: 3/5

The port expresses the polynomial rerooting clearly: `convolve` combines independent branches and `hung` adds the choice of the top city. The friction's `## Dream` is the port with one line changed (exact division in place of `without`), so the port is judged against that; the reported division gap is the one sanctioned deviation, but the child-product fold is written twice and an unused binding remains.

## Checklist failures
- idiomatic-code/R4, unreported: the same child-product fold appears at L50, `for c in tree.children(v): inside = convolve(inside, down[c])`, and L58, `for c in tree.children(v): around[v] = convolve(around[v], down[c])`.
- idiomatic-code/R13, unreported: L27, `let height: 0..=35 = V.map(v => tree.depth(v)).max()!`, is never used; it only asserts the depth bound.

## Owed rewrites
- Keep each vertex's child product from the bottom-up pass and seed `around[v]` from it, removing the second fold at L58.
- Drop `height` (L27), or move the depth bound into the header if it matters for the cost.
- The friction's "no exact series division for 1e9+7" claim has no `cpl run`; add one for the dream line `around[p] / down[v]` (idiomatic-code/R11).

## Language gaps
- Exact power-series division over a non-NTT modulus would replace `without` (L33–37); in the friction, without the R11 run.

## Dream notes
The dream line `if let p = tree.parent(v): around[v] *= hung(around[p] / down[v])` names the algebra, but read as a patch on the port it inherits the duplicated child-product fold (L50, L58), so 3.
