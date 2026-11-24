# Friction report: `cpl fmt` across the corpus

`cpl fmt` was run on every corpus `port.cp`, `dream.cp` and `spec.cp` that `cpl fmt --check`
flagged: 263 files (cf2237i1 and cf2237i2's dreams wait on a dream fix). `scripts/fmt_check.py` found no meaning change, no non-idempotent file and
no formatter bug in them. fmt's output is kept everywhere, even where it reads worse than the
input. Each pattern below has a minimal repro; run it with `cpl fmt --stdin < repro.cp`.

## A long list literal of short rows goes one element per line
- **Where:** egoi22p8/port.cp `orbits` (7 pairs in 2 rows becomes 7 lines), noi21p5/port.cp
  (`dot(...)` products 2 per row), and ioi25_migrations' case tables (24 pairs in 5 rows).
- **Repro:**
  ```cp
  let segs: [0..12]((Int, Int), (Int, Int)) = [
      ((2, 0), (2, 1)), ((2, 1), (2, 2)), ((1, 1), (2, 1)), ((1, 1), (2, 0)),
      ((1, 2), (2, 1)), ((0, 0), (2, 1)), ((0, 0), (1, 2)), ((0, 1), (1, 1)),
      ((0, 2), (1, 2)), ((1, 0), (2, 0)), ((0, 0), (0, 1)), ((0, 1), (0, 2)),
  ]
  print(segs[0].0.0)
  ```
  The output has 12 lines of one pair each.
- **What felt wrong:** a table laid out in rows (a 2x2 product, an editorial's case table)
  loses the shape a reader checks it by, and the file grows 3-5x in lines.
- **Goal it hurts:** readability of data tables.
- **Ideal code:** the input, unchanged.
- **General fix (fmt):** when a bracketed list doesn't fit on one line and the input already
  holds several elements per line, keep the rows, re-filling each up to the width (rustfmt
  and gofmt keep "short array element" grids this way).
- **Severity:** medium.

## A multi-line `if ... : a else: b` expression is joined past the width
- **Where:** noip04p4/port.cp `plausible`: three lines become one 173-character line.
  996warmup8p3/dream.cp: a two-line comprehension becomes one line over 100.
- **Repro:** `git show 46b28250d:corpus/problems/noip04p4/port.cp | cpl fmt --stdin | awk 'length > 100'`
  prints line 31 (173 characters).
- **What felt wrong:** the formatter's own width is broken, and the condition, the then
  branch and the else branch run together on one line.
- **Goal it hurts:** readability.
- **Ideal code:** the `if let ...:` header, the then expression and `else: false` each on
  their own line, as written.
- **General fix (fmt):** treat an inline `if`/`else` expression as breakable at `:` and
  `else:`, and never emit a line past the width when a break exists.
- **Severity:** medium.

## A `+` chain breaks inside its last operand instead of at the operators
- **Where:** egoi22p8/port.cp `below`; the closing `)` there also lands below the
  continuation indent.
- **Repro:**
  ```cp
  fn f(i: Int) -> Int = i
  fn below(a: Int, b: Int, c: Int, d: Int) -> Int:
      sum(f(i) for i in 0..a) + sum(f(i + a) for i in 0..b) +
          sum(f(i + b) for i in 0..c) + sum(f(a + b + c + i) for i in 0..=d)
  print(below(1, 2, 3, 4))
  ```
  The output joins the four sums onto one line ending in `sum(` and puts the last sum's body
  on the next line.
- **What felt wrong:** the terms of a sum are the unit a reader scans, and the break splits
  one of them.
- **Goal it hurts:** readability.
- **Ideal code:** the input, or one term per line after a leading break.
- **General fix (fmt):** prefer breaking a binary-operator chain at its operators (all of
  them, at one indent) before breaking inside an operand's brackets.
- **Severity:** low.

## A range breaks inside its end's call instead of at `..=`
- **Where:** ccc14s4hard/port.cp `strips`.
- **Repro:**
  ```cp
  let size = 3
  domain Strip = 0..10
  domain Block = 0..4
  fn strips(b: Block) -> Range<Strip> = Strip.nth(b.ord * size)..=Strip.nth(min(Strip.len, (b.ord + 1) * size) - 1)
  print(strips(Block.first!).len)
  ```
  The output is `... = Strip.nth(b.ord * size)..=Strip.nth(` with the end's argument on the
  next line.
- **What felt wrong:** the two ends of the range are what a reader compares, and the break
  falls in the middle of one of them.
- **Goal it hurts:** readability.
- **Ideal code:** a break after `=` (or before `..=`), with each end whole.
- **General fix (fmt):** rank a break after a `fn ... =` or before a range operator above a
  break inside a call's parentheses.
- **Severity:** low.

## Trailing-comment alignment pushes a comment past the width
- **Where:** btoi18p4/port.cp:55, ioi08p3/port.cp:31 and noi05p1/port.cp:37 now run past
  100 columns. Each comment moved right to line up with a neighbouring line's comment.
- **Repro:**
  ```cp
  let order = [3, 1, 2].sorted(by: w => (w, desc(w)))  // by start, longest first
  for s in [0, 3]:  // the circle twice, so a wire past n can hold the ones it wraps over
      print(s)
  ```
  The second line becomes 122 characters long, with its comment moved to column 55.
- **What felt wrong:** a line that fit is pushed over the width only to align it with its
  neighbour.
- **Goal it hurts:** readability.
- **Ideal code:** the input; alignment should give way when the aligned line would overflow.
- **General fix (fmt):** leave a line out of its comment-alignment group (keep two spaces)
  when aligning it would exceed the width.
- **Severity:** low.
