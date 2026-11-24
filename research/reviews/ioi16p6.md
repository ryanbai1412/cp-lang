# ioi16p6 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and `corpus/problems/ioi16p6/dream.cp`.

Reads like the editorial: outermost diagonal spans, the square of a run as a line in its left
end, and `aliens` over the photo fee with ties toward fewer photos. The dream uses
`MonotoneHull`; the port uses `LiChao` because `MonotoneHull` breaks ties the wrong way, a
reported stdlib bug with repro and measured cost.

## Checklist failures

- `let (lo, x) = (Int(s.l), Int(s.r))` (line 26) reads an inclusive span's ends with `.l`/`.r`
  where decision §101 names `.start`/`.end` (R8); the dream does the same.
- The header's `O(n log m)` (line 10) is one fee's cost; the fee search multiplies it by
  `log m^2` (ioi16p6io states it) (R1).

## Language gaps

- `MonotoneHull` tie-breaking: in research/frictions/ioi16p6.md.

## Dream notes

- At bar.
