# Friction report: ioi16p4

## Dream

```cp
let row = read<Str of "X_.">()
domain Cell = 0..row.len
let clues = read<[k]1..=n>()
domain Clue = 0..k
// fits[c, j]: the cells before cut c hold exactly the blocks before clue cut j
fn fits(cells: [Cell]Char, lens: [Clue]Int) -> [Cell.cuts, Clue.cuts]Bool
let before = fits(row, clues)
let after = fits(row.rev, clues.rev)    // a reversed view keyed by the same Cell / Clue
let whites = Prefix([Cell] c => Int(row[c] == '_'))
...
print(Str(Cell.map(c => if white(c) and can_black[c] > 0: '?' elif white(c): '_' else: 'X')))
```

## The two-sided DP lives in `Int` tables

- **Where:** `fn fits(row: Str, lens: List<Int>) -> [0..=n, 0..=k]Bool`, `after[n - end - 1, k - 1 - j]`.
- **What happened:** the DP runs once on the row and once on the reversed row. A `Str` has no
  `Cell`-keyed reversed view (`row.rev` is a fresh `Str` with its own `Int` indices), so the
  tables are keyed by `Int` cuts and every lookup into `after` mirrors the index by hand
  (`n - end - 1`, `k - 1 - j`), the arithmetic the domain was meant to remove.
- **General fix:** `a.rev` on a `[D]T` table (and a `Str` read as `[Cell]Char`) as a view keyed
  by `D` whose cuts mirror, so `after[c.mirror, j.mirror]` or simply `after[c, j]` reads the
  suffix DP in the forward coordinates.
- **Goal it hurts:** hard to get wrong. **Severity:** medium.

## Known-white counts are a hand prefix

- **Where:** `fn whites_before`.
- **What happened:** `Prefix` is keyed by a domain; over the `Int` index of a `Str` (and its
  reversal) the port keeps a hand table. Goes away with the view above.
- **Severity:** low.

## Measurements

Contestant-written tokens: port 537, original 521 (`corpus/audit.py` count). Emitted C++: 24306 bytes (`cpl build`, judge `generic`).
The accepted C++ is a grader function with no main (it ran 9.7 s on DMOJ), so the max input is timed only by `corpus/generators.py`.
