# Friction report: sdoi17r1p6

There is no dream.cp; the ideal is below. The port is a lazy `SegTree` of the four sums (xy, x, y, xx) under a set-then-add `Edit` action, with the slope taken in closed form and I128 sums. The max output differs from the C++ only below the checker's 1e-5 tolerance (`checker.py` accepts it).

## Dream

```cp
struct Sums(xy: I128, x: I128, y: I128, xx: I128)
fn edit(f: Edit, v: Sums, span: Range<Pos>) -> Sums: ...   // the action sees which cells it covers
```

## A set tag needs positions, so every node carries `first`
- **Where:** line 18: `struct Sums(..., first: Int)`. Line 23: `join` takes `min(a.first, b.first)`. Line 52: the identity uses `n + 1` as its `first`.
- **What felt wrong:** "set x[i] = S + i" depends on i, but `Action`'s apply sees only the node's length. The port stores each node's start in the monoid value, and `n + 1` is a sentinel start for the empty sum.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** the dream's `apply` receiving the node's span (`Range<Pos>`), so that `first` is not data.
- **General fix:** let `Action.apply` take the node's index range as well as its length; the segment tree already knows it. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 770, accepted C++ 1652 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.57 s / 36 MB, accepted C++ 0.33 s / 65 MB.
- **Emitted C++:** 30206 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
