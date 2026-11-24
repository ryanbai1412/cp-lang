# ioi11p3 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi11p3.md).

Reads like the editorial: fields as a `Field` domain over `Spot = 1..=1e9`, the block cost
from `prefix(x)` around the median `mid`, and a two-pointer window. The cost formula checks out:
`S(mid..=r) - S(l..=mid) - x[mid] * ((r - mid) - (mid - l))` is the sum of distances to
`x[mid]`. The window never passes `r` since `cost(r, r) = 0 <= budget`.

## Checklist failures

- `var l` and the `while cost(l, r) > budget: l = l.next!` loop hand-move the window (R3);
  reported.
- `sorted_fields` is generator scaffolding for a sorted input; reported.

## Language gaps

- A monotone longest-window search, a sorted-input type and a bound naming `len`: both in
  research/frictions/ioi11p3.md.

## Dream notes

- Short and named (`Field.longest_window`, `sorted [Field]1..=len`), but it leaves `cost`
  implicit; the port's `cost` is what the dream would keep.
