# cco22p1

Score: 4/5
Dream: 4/5

`consistent` builds the alternating taller-than graph and asks std `topo`; `reach` grows by two pointers, then each query is O(1). The header says why shrinking keeps a range valid and why two pointers beat the measured binary search.

## Checklist failures

None beyond the reported gap.

## Owed rewrites

- None.

## Language gaps

- No `enumerate` on `adjacent`, so the parity is `(i.ord - x.ord) % 2` (E0203 quoted): research/frictions/cco22p1.md.

## Dream notes

- The dream is `consistent` with `adjacent.enumerate`; the rest is the port.
