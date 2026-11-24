# iati21a22

## Dream

```cp
let order = sorted((u: p.x + p.y, v: p.x - p.y) for p in pts, by: p => (desc(p.u), desc(p.v)))
print(order.lis(by: p => p.v).len)
```

## Descending keys are negated
- **Where:** the sort key `(-p.u, -p.v)`; `desc(..)` keys would say the order
  instead of relying on the values being negatable.
- **Severity:** low.

## The statement is not in the repo
- **Where:** the header's Problem and the bounds follow the earlier port and the
  accepted C++; DMOJ serves the statement behind a browser check.
- **Severity:** medium.
