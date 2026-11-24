# yac6p5

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against the ideal under `## Dream` in `research/frictions/yac6p5.md`
(there is no `dream.cp`), without the C++. The header derives the key fact (V is one bit per
axis and a turn flips one, so the axes are independent), the split into a low and a high group
gathered at their medians, the turn count, and the cost. `axis` is that argument: sort, prefix
sums through `pre.fold`, a median `gather`, and one sweep over the split point.

## Checklist failures
- idiomatic-code/R2: the quadrant is decoded with `s.v >= 2` and `s.v == 1 or s.v == 2` (L36)
  instead of named directions. Reported in the friction; re-run below.

## Owed rewrites
- None today.

## Language gaps
- Reading an enum by its ordinal (in the friction). Fresh `cpl run` (2026-11-24):
  `let q: Quadrant = read()` is E0302 "reading a `Quadrant` needs to know how its variants are
  written in the input"; `Quadrant.at(read<0..=3>())` gives `?Quadrant`, but adding `!` is
  E0302 "`!` takes the value out of an optional, but this is a `Quadrant`", which contradicts
  that type. Both added to the friction.

## Dream notes
The dream keeps the port's axis algorithm and names the quadrant as an enum with direction
predicates; it is a sketch on top of the port rather than a full program, so 4/5.
