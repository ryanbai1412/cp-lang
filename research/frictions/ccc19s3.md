# Friction Report: ccc19s3

## Retrying a randomized attempt
- **Where:** `var found = attempt()` / `while found.is_none: found = attempt()` / `print(found!)`.
- **What felt wrong:** "run until it succeeds" needs a mutable optional and an unwrap.
- **Goal it hurts:** brevity; common mistakes hard to write.
- **Ideal code:** `print(retry(attempt))` or `print(iterate_some(attempt))`.
- **General fix:** a std `retry(f: () -> ?T) -> T` (or `repeat_until_some`) in `std.seq`.
- **Severity:** low

## Lines of a grid as cell tuples
- **Where:** `let lines: List<(Cell, Cell, Cell)> = [((i, Ax(0)), ...) for i in Ax] + ...`.
- **What felt wrong:** lists can't be unpacked, so each line is spelled as a 3-tuple of cells
  by hand, and indexing takes `sq[a.0, a.1]` rather than `sq[a]`.
- **Goal it hurts:** brevity.
- **Ideal code:** `for (a, b, c) in sq.rows + sq.cols` with cells usable as 2-D indices.
- **General fix:** `rows`/`cols` views on 2-D arrays and indexing by a `(A, B)` cell.
- **Severity:** low

## Reading a marker token as `None`
- **Where:** `fn read_cell() -> ?Int` (`let w: Str = read()`; `if w == "X": None else: Some(Int(w))`)
  and the double loop that fills `given`.
- **What felt wrong:** a grid of numbers with a placeholder token cannot be read in one line.
  `cpl run` of `let given: [Ax, Ax]?Int = read(none: "X")` fails with
  `error[E0302]: expected Int, found Str` (at `"X"`).
- **Goal it hurts:** quick to write (input parsing is boilerplate).
- **Ideal code:** `let given: [Ax, Ax]?Int = read(none: "X")`.
- **General fix:** a `none:` token option on `read` for `?T` targets, applied element-wise.
- **Severity:** low
- **Owner:** stdlib.

## Dream

```cp
use std.rand.{Rng}
domain Ax = 0..3
let given: [Ax, Ax]?Int = read(none: "X")
fn settle(mut sq: [Ax, Ax]?Int) -> Bool:
    while let (cell, v) = (sq.rows + sq.cols).find_map(forced):
        if v.is_none: return false
        sq[cell] = v
    true
var rng = Rng.new(2019)
print(retry(() =>
    var sq = given.clone()
    while settle(mut sq) and sq.cells.any(.is_none): sq[rng.pick(sq.keys(.is_none))!] = Some(rng.int(-500, 499))
    sq.all_some()))
```

## Measurements

- Tokens: port 514, original 1086
- Max input (`gen.py --max`): port 0.00 s, 3072 KB, original 0.00 s, 3456 KB
- Emitted C++: 20960 bytes
