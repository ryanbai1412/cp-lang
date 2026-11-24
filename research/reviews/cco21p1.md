# cco21p1

Score: 2/5
Dream: 3/5

The inversion-count argument is clear: std `buckets` and `inversions`, and per swap `2 b(x, y) - |x||y|`. The memo is a hand `Map` because the friction says there is no `@memo`, and there is.

## Checklist failures

- idiomatic-code/R13: `known` and its lookup/store (lines 21, 25, 29-30) hand-roll `@memo`: a recursive `@memo fn before(x: V, y: V) -> Int` reading a global table runs under 0.2.
- idiomatic-code/R6: `Val(Int(s))` (line 34) crosses `Slot` to `Val` by ordinal; reported.

## Owed rewrites

- `@memo fn before(...)`; drop `known` and the friction's first bullet (measure, since `@memo` on a `(Val, Val)` key may differ from the `Map`).

## Language gaps

- A `Slot` is not a `Val`: research/frictions/cco21p1.md.

## Dream notes

- The fragment uses `x` and `y` in the loop without binding them (idiomatic-code/R13, caps at 3).
