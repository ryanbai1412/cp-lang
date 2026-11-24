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

## Two optional minima can't be combined (fixed)
- **Status:** fixed. `min(?a, ?b)` skips `None` today: a fresh `cpl run` of
  `let a: ?Int = None; let b: ?Int = Some(3)` then `print(min(a, b) ?? -1)` and
  `print(min(b, a) ?? -1)` prints `3` twice. The port still combines them by hand,
  `min(by_row ?? by_col!, by_col ?? by_row!)` (port lines 27-29). Rewritten as
  `min(by_row, by_col)!`, it gives the same output on tests.toml, so this is an
  owed port rewrite.
- **Severity:** low.
