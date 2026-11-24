# cco19p5

Score: 4/5
Dream: 4/5

The header's proof in six lines: each vertex counts its higher neighbours, hands the rest to the lowest one, and the sets merge small to large with `absorb`. The port is the dream; what stays is speed.

## Checklist failures

- idiomatic-code/R10: 0.60 s against 0.06 s for the C++ (10x); reported with the measurement.

## Owed rewrites

- None in the port.

## Language gaps

- `SortedSet.absorb` is 10x the `std::set` merge: research/frictions/cco19p5.md.

## Dream notes

- `dream.cp` is the port with a header; `out[i].remove(i)` handles the vertex absorbed into its own set where the port removes `f` first. Either reads as the editorial.
