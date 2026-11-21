# 27. Algebra and segment trees

**Layer:** library. `Monoid`, `Group`, `Action` and their laws are traits; `SegTree` and `Fenwick` are library structures over `[D]T`. `cp test` samples the laws. Status: [proposed].

## 27.1 The form

```cp
var t = SegTree<D, M, A>(init: values)   // `init`: a `[D]M`, or one `M` copied to every position
t.fold(r)                                // M's fold over a range, a domain (whole) or a position
t.apply(r, tag)                          // A's action over the same
t[i] = value                             // point assignment
t.prefix_fold(start: c, while: p)        // (end cut, fold) of the longest run from cut c (default first) whose fold satisfies a monotone p (max_right) [p1 F11, r13 P15, p2 F6]
t[i]                                     // leaf read: O(1) without `A`, else O(log n) [r14 T4]
```

- `fold`/`apply` take exactly what array slicing takes (21.4), cut ranges included [p2 S7]. `A` may be omitted
  (`SegTree<P, Sum<Int>>`) for point assignment only. `Assign<M>` requires `M: Idempotent` (law sampled by
  `cp test`) or a `repeat(x, len)` method. `Fenwick<D, M>` is explicit: `M: Monoid` gives
  prefix folds, `M: Group` adds range folds [r14 T19].
- `D` is any `Index` (chapter 13.2). `M` implements `Monoid`; `A` implements `Action<M>`.
- Tags are values: `Add(x)`, `Assign(x)`.
- `Max<T>` and `Min<T>` are monoids on `?T` with identity `none`, so no `Inf` sentinel overflows [r3].
- Stdlib provides `Sum`, `Max`, `Min`, `MaxCount`, `MinCount`, `Add`, `Assign`, `Affine`; anything else is ~15 lines of user algebra.

## 27.2 Algebra as traits

```cp
impl Monoid for Cover:
    id = Cover(low: none, at_low: 0, hist: 0)
    fn op(l, r) = …
impl Action<Cover> for Tick:
    id = Tick()
    fn then(t, u) = …                        // t first, then u
    fn apply(t, x) = …
    fn restrict(t, child, scope) = …         // optional; absent means `t`
    fn uniform(t) -> Bool = …                // optional; absent means false
```

No `monoid`/`action` keywords: these are traits like `Index` [r10 1, 43].

**Laws** [r10 17], with `P = op(l, r)`:
- `op` is associative with identity `id`; `then` is associative with identity `id`.
- `apply(t, op(l, r)) == op(apply(restrict(t, l, P), l), apply(restrict(t, r, P), r))`.
- `apply(then(t, u), x) == apply(u, apply(t, x))`.
- `uniform(t)` implies `restrict(t, c, s) == t`.

`cp test` checks every law on random values and `apply` on random ranges against the meaning below, on
non-power-of-two sizes.

## 27.3 Restricted actions

Some tags affect only the elements that match the range's fold, e.g. "add 1 to every element equal to the
range minimum" (Beautiful Subsegments).

- **Meaning:** `apply(R, t)` sets each element `e` in `R` to `apply(restrict(t, e, S), e)`, where `S` is
  `fold(R)` *before* the tag. The tree must agree whatever nodes `R` splits into, so it folds `R` first and
  restricts every canonical node against `S` [r3].
- **Cost:** one extra `fold`. Uniform tags skip it, which is free when the tag is a literal [r4 8, 9].
- A caller that already holds the fold passes it: `apply(R, t, scope: s)`, checked in debug [r5 8].
- `cp explain` shows which `apply` calls pay the extra fold.
- Segment tree beats need a `try_apply` hook; deferred until the ports need it.

## 27.4 Lowering

Iterative bottom-up lazy tree, size `2·2^⌈log₂ n⌉`; values and tags in separate arrays; tags only on
internal nodes; `?Int` values with a niche. Measured on Beautiful Subsegments (n = 2·10⁵, q = 10⁶): equal or
faster than a conventional recursive tree, memory 0.73–1.06x (`lower/beautiful/RESULTS.md`).

## 27.5 Round-3 rules [p3]

- An action with no `restrict` is uniform (`restrict = t` is trivially uniform), so `apply` never pays the extra
  fold. This replaces 5.2's "`uniform` absent means false" for actions without `restrict`.
- `Fenwick<D, M>(init:)` has `f.add(i, x)` and `f.fold(.. c)`; `M: Group` adds any range.
- A `SegTree` without `A` allows `t[i] op= x`.
- Node types may hold small static arrays (13.4 storage rule).
