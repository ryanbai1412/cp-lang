# Friction Report: cco18p5

The v0.2 `Peak(has: Bool, val, idx: -1)` sentinel struct is now `?Peak` with `Peak = (val, at: I)`,
the bounds are types (`2 <= K <= N`), and the arrays are read straight into `[I]`.

## Arg-max needs a hand-written Option monoid
- **Where:** `fn better(x: ?Peak, y: ?Peak) -> ?Peak`.
- **What felt wrong:** "the largest, later on ties" is `max` over `(val, at)`, but a SegTree needs a
  monoid with an identity, so the `None` wrapping and the comparison are spelled out.
- **Ideal:** `SegTree<I, ?Peak>(max)` lifting a semigroup to an Option monoid, or a std
  `ArgMax<I, Int>` monoid.

## "Within k - 1 of i, on one side" takes index arithmetic
- **Where:** `left`, `right` with `I.nth(max(0, i.ord - k + 1))` and `i.next`.
- **Ideal:** `i.before(within: k - 1)` / `i.after(within: k - 1)` as clamped domain ranges.

## Dream
```cp
let n: 2..=1e6 = read()
let k: 2..=n = read()
let q: 0..=1e5 = read()
domain I = 1..=n
var a: [I]1..=1e9 = read()
let upds: [q](pos: I, x: 1..=1e9) = read()
var peaks = SegTree<I, ?(Int, I)>(max, [I] i => Some((a[i], i)))
var pairs = SegTree<I, Int>(max)
fn refresh(i: I):
    let p = max(peaks.fold(i.before(within: k - 1)), peaks.fold(i.after(within: k - 1)))
    pairs.set(i, if let (v, j) = p and (a[i], i) > (v, j): a[i] + v else: 0)
for i in I: refresh(i)
print(pairs.fold(I.all))
for u in upds:
    let olds = [peaks.fold(u.pos.before(within: k - 1)), peaks.fold(u.pos.after(within: k - 1))]
    (a[u.pos], peaks[u.pos]) = (u.x, Some((u.x, u.pos)))
    refresh(u.pos)
    for (_, j) in olds.flatten: refresh(j)
    print(pairs.fold(I.all))
```

- Tokens: port 469, original 1530
- Max input (`gen.py --max`): port 0.40 s, 60908 KB, original 1.90 s, 228712 KB
