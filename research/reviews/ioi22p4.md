# ioi22p4

Score: 4/5
Dream: 4/5

Cold review: the linearity argument in code: `ways` bottom-up, `worth` top-down with all-but-one-child products, then a lazy `SegTree` of `Flip<Z>` over the sources whose root holds the lit total (lines 29-55).

## Checklist failures

- R6: `worth[Node(Int(s))]` (line 49) crosses from `Source` to `Node` through `Int`; `Source = n..n + m` is a stretch of `Node` but not typed as one.

## Owed rewrites

- None.

## Language gaps

- `act: flip` not wired, `Flip.op(+)` lowering to `std::function`, prefix/suffix products by hand (lines 38-43), gate passes relying on `P[i] < i`, the grader entry: all in research/frictions/ioi22p4.md.
- A domain that is a range of another (`Source` inside `Node`) is not reported; filed in research/frictions/ioi22p4.md.

## Dream notes

- `prod_except(ways, tree.children(u), v)` is per child, so as written it reads as O(deg^2); the dream should say it is the prefix-times-suffix product.
