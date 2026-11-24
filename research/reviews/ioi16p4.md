# ioi16p4 review

Score: 3/5
Dream: 3/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi16p4.md).

The two-sided `fits` DP, the difference array over placements and the white/black test follow
the header. But the DP lives in `[0..=n, 0..=k]Bool` with `Int` cuts on both axes, read back
mirrored as `after[n - end - 1, k - 1 - j]`, and known-white counts are a hand prefix. Both
are reported.

## Checklist failures

- `fits(row: Str, lens: List<Int>) -> [0..=n, 0..=k]Bool` and the mirrored indices in lines
  55 and 63 (R6); reported.
- `whites_before` (lines 21-24) is a hand prefix count (R3); reported.
- `let start = if j > 0: i - lens[j - 1] else: -1` then `start >= 0` (lines 35-36) uses `-1`
  for "no block" (R5); part of the reported `Int` tables.

## Language gaps

- A reversed view keyed by the same domains and `Prefix`: in research/frictions/ioi16p4.md.

## Dream notes

- Names the right pieces (`Cell.cuts`, `Clue.cuts`, a reversed view, `Prefix`), but `fits`
  has no body and the placements are `...`: the DP, which is the hard part, is not shown. It
  should spell `fits` over `[Cell.cuts, Clue.cuts]` and the placement loop.
