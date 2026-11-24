# egoi23p4

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`. Both lane matrices in bike-lane terms as `[V, V]?Int`, closure over `Semiring(max, min)` and `Semiring(min, max)`, and the union of a minimum and a maximum `kruskal` forest: the header's argument.

## Checklist failures
None remaining. The port checked both full matrices as well as the offered streets; when the offers close back to a matrix that matrix is closed, so the two extra O(n^3) closures were redundant (R4). Removed in this review: 16/16 cases pass, and the max input runs in 0.20 s, down from 0.37 s (accepted C++ 0.37 s).

## Owed rewrites
None.

## Language gaps
None.

## Dream notes
The dream builds the same two forests through a generic `rebuilds<R>`; the port is it inline.
