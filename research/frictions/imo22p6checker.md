# Friction Report: imo22p6checker

## Dream

```cp
let a: [Row, Col]Value = read()
for p in a.inverse():                     // cells in increasing value
    let lower = p.adj4.filter(q => a[q] < a[p])
    ways[p] = if lower.is_empty: 1 else: lower.sum(q => ways[q])
```

## Gaps

- **`inverse()` rejects a grid of a domain.** `[Row, Col]Value.inverse()` is
  E0302 ("requires elements of an index domain or bounded range"), even
  though `Value` is a domain and the grid is a bijection onto it. The port
  fills `at: [Value]Cell` by hand, starting from a placeholder cell.
  Severity: medium.
