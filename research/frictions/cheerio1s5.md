# cheerio1s5 frictions

## Dream

```cp
@memo(array)
fn clear(l: S.cuts, r: S.cuts) -> Int = max(r - l + clear(l, i.l) + clear(i.r, r) for i in S.between(l, r) if s[i] == 'H') ?? 0
```

## Integer cuts

The interval DP runs over Int cuts `0..=n`: there is no spelling for the elements between two cuts,
so the cut-typed memo above does not build, and `Int(Bool)` has no named-function form (`head.map(Int)`).

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
let s: [domain S]Char = read()
@memo(array)
fn clear(l: S.cuts, r: S.cuts) -> Int = max(r - l + clear(l, i.l) + clear(i.r, r) for i in S.between(l, r) if s[i] == 'H') ?? 0
print(clear(S.cuts.first!, S.cuts.last!))
```

```text
error[E0203]: a `domain S` has no method `between`
  --> probe.cp:3:94
3 | fn clear(l: S.cuts, r: S.cuts) -> Int = max(r - l + clear(l, i.l) + clear(i.r, r) for i in S.between(l, r) if s[i] == 'H') ?? 0
  |                                                                                              ^^^^^^^
```

**Status:** open.
