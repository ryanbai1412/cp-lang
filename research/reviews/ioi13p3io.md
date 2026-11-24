# ioi13p3io review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi13p3io.md, which points to the ioi13p3 design in
`corpus/problems/ioi13p3/dream.cp`).

Reads like the editorial: `?Cost` as the (min, +) zero, the Knuth-bounded product with `opt`,
blocks rebuilt on change, the root read on escape. The remaining gaps (blocks spelled with
cuts, `Col(g.ord + 1)`, `map(..) ?? true`, 8-byte `?Cost` cells forcing 20-row blocks) are
reported.

## Checklist failures

- The header says blocks are built "in O(10 C^2)" (line 8) while `BLOCK = 20` (line 15); the
  cost is O(BLOCK C^2) per block (R1).
- `routes.set(block_of(x.ord), block(block_of(x.ord)))` is written once per update kind
  (lines 75, 79) (R4).

## Owed rewrites

- One `fn refresh(row: Int)` computing the block once, as the dream does.

## Language gaps

- All in research/frictions/ioi13p3io.md.

## Dream notes

- The friction's dream is prose pointing at ioi13p3's dream.cp; see research/reviews/ioi13p3.md.
