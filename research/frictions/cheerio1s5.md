# cheerio1s5 frictions

## Dream

```cp
@memo(array)
fn clear(l: S.cuts, r: S.cuts) -> Int = max(r - l + clear(l, i.l) + clear(i.r, r) for i in S.between(l, r) if s[i] == 'H') ?? 0
```

## Integer cuts

The interval DP runs over Int cuts `0..=n`: there is no spelling for the elements between two cuts,
so the cut-typed memo above does not build, and `Int(Bool)` has no named-function form (`head.map(Int)`).
