# ioi09p4: Raisins

## Dream

```cp
@memo
fn pay(r1: Row, r2: Row, c1: Col, c2: Col) -> Int:
    if r1 == r2 and c1 == c2: return 0
    pre.fold(r1..=r2, c1..=c2) + min(
        (r1..r2).min(r => pay(r1, r, c1, c2) + pay(r.next!, r2, c1, c2)),
        (c1..c2).min(c => pay(r1, r2, c1, c) + pay(r1, r2, c.next!, c2)),
    )!
```

## Two optional minima can't be combined
- **Where:** `pay` builds both cut lists and concatenates them to take one
  `min()!`; a `min` over optionals that skips `None` would avoid the lists.
- **General fix:** `min(?a, ?b) -> ?T` (shared with ioi04p4).
- **Severity:** low.
