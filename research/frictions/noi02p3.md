# Friction report: noi02p3

spec.cp stopped checking: its edge record used `a`/`b` (Graph builders read `u: V, v: V`), it
read the parent branch with `t.up_arc(c)!.cost`, and it merged children with
`convolve(List<?Int>, over: Semiring(min, +), shape: Curved.Neither)`. It now walks `g[v]`
(skipping the arc back to the parent) and writes the min-plus merge as a `min=` loop over lifted
`?Int` `+`.

## A `?Int` list can't be `convolve`d over `Semiring(min, +)`

- **Where:** `let via = (w, wc) => convolve(best[w], sub[wc], over: tropical, shape: Curved.Neither)`
- **What felt wrong:** R02.4.4 says lifted arithmetic is three ordinary impls
  (`impl<T: Add> Add for ?T`), so `None` as "unreachable" should be a `T` for any `T: Add` bound.
  Today `?Int + Int` works as an operator but `?Int` satisfies no `Add` bound.
- **Repro:** `fn twice<T: Add>(a: T) -> T = a + a` / `let x: ?Int = Some(3)` /
  `print(twice(x) ?? -1)` -> E0606 "`?Int` does not implement `Add` required by `twice`".
- **Goal it hurts:** express (the tree knapsack merge is the textbook min-plus product).
- **Ideal code:** `convolve(best[w], sub[wc].map(x => x + cost), over: Semiring(min, +), shape: Curved.Neither)`.
- **General fix:** make the R02.4.4 lifts real impls (or have bound checks see them), so every
  generic over `Add`/`Sub`/`Mul` (convolve, Mat, prefix folds) takes `?T`.
- **Severity:** medium (a 4-line loop replaces it)
- **Status:** routed to compiler (`plan/domains/compiler/worklog.md`).

## `Rooted` drops arc fields

- **Where:** `t.up_arc(c)!.cost`, and `t.child_arcs(v)` with `a.cost`.
- **What happened:** `up_arc` is `todo` in std/tree.cp, and the built-in `Rooted` types
  `child_arcs` items as `Int` (E0203 "an `Int` has no member `cost`"), so spec.cp skips `Rooted`
  and recurses over `g[v]` with the parent passed down.
- **Severity:** low
- **Status:** already owned (`std/GAPS.md` tree `Rooted` entries).

Measurements (n = 300, 3 heads, k = 150): `cpl run` 0.47 s / 55 MB; the emitted C++
(18,453 B, g++ -O2) runs in under 0.01 s / 3 MB, the AC submission 3718952 in 0.03 s / 4 MB.
