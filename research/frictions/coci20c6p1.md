# coci20c6p1 frictions

## Dream

```cp
let out = [Row, Col] (r, c) => [(r, c), (r, c).up, (r, c).left, (r, c).up?.left].any(p => p.map(q => g[q] == '#') ?? false)
```

## Optional neighbours

The bold square reaches right and down; each neighbour is an optional `Col.at`/`Row.at`, so
the port marks the three cells with nested `if let`s.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
domain Row = 0..2
domain Col = 0..2
let p = (Row.last!, Col.last!)
print(yes(p.up?.left != None))
```

```text
YES
```

**Status:** fixed: `std.grid` has optional `up`/`left` on a cell, so the dream's gather builds; the port's nested `if let` scatter is owed.
