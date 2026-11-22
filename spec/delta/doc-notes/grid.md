# Doc notes: `std.grid`

- `Dir4.from`: chose to trap when `letters.len != 4`, and to take the first
  matching index when a letter repeats; the spec gives only the two examples.
- `Dir4.from`: matching is exact (case-sensitive); `'u'` is not `'U'`.
- `adj4`, `adj8`, `ray`, `step`: off-grid cells are skipped (`None` for `step`),
  never trapped; the order is the documented clockwise order with gaps.
- Contradiction: decision 16.5 says `g.transposed` (and `g.flipped`) are
  zero-copy views; the stub (and decision 18.5's `[C, R]T`) make `transposed`
  an O(rows · cols) fn returning an owned grid. Documented the stub.
- Contradiction: decision 16.5 lists `.from("UDLR", ch)` for `Dir8` too; the
  stub has no `Dir8.from` (decision 7.19 demotes `Dir8`). Left as is.
- Gap: `Dir8` has no named constructors (`up()`, ...), so its examples index
  `Dir8.all()`.
