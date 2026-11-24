# Friction Report: ioi07p5

## Dream

```cp
pairs += near.fold(a.v.within(d))   // the positions within d of a.v, clipped to V
```

## Gaps

- **No clipped "within r of x" range on a domain.** The box round a turned
  animal is `(x.prev(r) ?? V.first!)..=(x.next(r) ?? V.last!)`, three times;
  `V.clip` takes only a `Range<V>`, so an integer window can't be clipped
  straight into the domain. Severity: medium.
- **Reads depend on the board type.** Each of `line`, `grid` and `cube` reads
  its own `[n]` tuple shape, because `dims` decides how many coordinates an
  animal has. Severity: low.
