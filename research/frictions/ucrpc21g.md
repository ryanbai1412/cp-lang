# Friction report: ucrpc21g

There is no dream.cp; the ideal is below. The port keeps the most popcorn held per cell (`[Row, Col]?Int`, `None` while unreachable). Each second it takes the best of the cell and its `adj4` neighbours, then applies that second's drops from the `gain` and `crush` tables.

## Dream

```cp
let popcorn: [0..coins](at: Cell, t: Tick) = read()
...
held = [Cell] p => moved[p].map(h => if crush[t, p]: h / 2 else: h + gain[t, p])
print(held.values().present().max()!)
```

## A product-domain cell is split by hand to index a wider array
- **Where:** line 39: `crush[t, p.0, p.1]` and `gain[t, p.0, p.1]`. Line 41: `[held[r, c] for r in Row for c in Col]`. Drops are read as flat `(Row, Col, Tick)` triples.
- **What felt wrong:** `Cell = (Row, Col)` indexes `held` directly, but a `[Tick, Row, Col]` array needs the tuple taken apart. "Every value of a 2-D array" is a nested comprehension.
- **Goal it hurts:** express.
- **Ideal code:** the dream's `crush[t, p]`, with a tuple of domains standing for its components in an index list, and `held.values()`.
- **General fix:** let a product-domain value fill the matching run of an array's index positions, and give multi-dimensional arrays a `values()` iterator. Owner: core/spec (index splatting), stdlib (`values`).
- **Severity:** low.

## Measurements

- **Tokens:** port 298, accepted C++ 627 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.21 s / 5 MB, accepted C++ 7.94 s / 291 MB, with the same output.
- **Emitted C++:** 22912 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
