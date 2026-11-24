# ioi09p8: Salesman

## Dream

```cp
type Money = -1e10..=1e10
var from_above = SegTree<Loc, Money>(max)   // empty fold = Money's low end
fn reach(x: Loc) -> Int = max(from_above.fold(..=x) - down * x.label, from_below.fold(x..) + up * x.label)
fn stand(x: Loc, money: Int):
    from_above.at(x) max= money + down * x.label
for day in fairs.sorted_in_place(by: f => (f.day, f.at)).runs(by: f => f.day):
    let going_down = day.carry(f => reach(f.at) + f.profit, (prev, f, g) => prev - down * (f.at - g.at) + f.profit)
    ...
```

## A bounded max SegTree folds the empty range to Int's low end
- **Where:** `from_above`/`from_below`. `SegTree<Loc, Money>(max)` with
  `type Money = -1e10..=1e10` folds an empty range to `-2^63 + 1`, not -1e10,
  though the docs say the identity is the low end of `T`, so `fold - down * x`
  overflows. The port stores `?Int` and unwraps once in `reach`. Repro sent to Core.
- **Severity:** medium.

## A carried sweep over a slice is an index loop
- **Where:** `going_down`/`going_up`: each value depends on the previous one and
  the gap to the previous fair. `scan` folds same-typed values only, so the two
  sweeps are index loops over a list built per day.
- **Severity:** low.
