# coci20c6p1 frictions

## Dream

```cp
let out = [Row, Col] (r, c) => [(r, c), (r, c).up, (r, c).left, (r, c).up?.left].any(p => p.map(q => g[q] == '#') ?? false)
```

## Optional neighbours

The bold square reaches right and down; each neighbour is an optional `Col.at`/`Row.at`, so
the port marks the three cells with nested `if let`s.
