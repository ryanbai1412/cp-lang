# ioi13p3 review

Score: 2/5
Dream: 4/5

Scored cold against the checklist, the port and `corpus/problems/ioi13p3/dream.cp`.

The structure is the editorial's: per-block (min, +) matrices in a `SegTree<Block, Matrix>`
with the Knuth-bounded product. But this port keeps `NONE = 1 << 30` as "no route" where its
dream and its stdin twin ioi13p3io use `?Cost`, and the friction report describes the `?Cost`
form (`map(..) ?? true`, `[Col, Col]?Cost` taking 8 bytes) rather than this port, so the
sentinel is unexplained here.

## Checklist failures

- `const NONE = 1 << 30` and `type Matrix = [Col, Col]0..=NONE` (lines 29-30) encode absence
  as a number (R5). ioi13p3io fits the memory limit with `?Cost` and 20-row blocks, so this is
  owed, not a language gap, and unreported: 2/5.
- `type Cost = 0..=1e7` (line 17) is declared and never used (R13).
- The header says blocks are built "in O(10 C^2)" (line 8) while `BLOCK = 20` (line 15); the
  cost is O(BLOCK C^2) per block (R1).
- The refresh `routes.set(block_of(..), block(block_of(..)))` then `root = routes.all` is
  written twice, once per update kind (lines 77-78, 82-83), and `root` is a hand-kept copy of
  `routes.all` (R4); the dream's `refresh(row)` says it once.

## Owed rewrites

- Take the ioi13p3io matrix (`[Col, Col]?Cost`, `if let via = x[i, k] + y[k, j]`), drop
  `NONE` and the unused `Cost`, and fold the two updates into one `refresh(row)` reading
  `routes.all` at the query.

## Language gaps

- The stateful grader object, blocks of a domain spelled with cuts, `Col(g.ord + 1)` for a
  gap's neighbours: in research/frictions/ioi13p3.md. Its first section still cites
  `proto/ports/scout_batch_52/` paths that no longer exist.

## Dream notes

- At bar for the structure: `?Cost` as the semiring zero, `refresh`, a long header deriving
  the Knuth bound. `solve_block` filters every `Row` per block
  (`Row.filter(row => row.ord / B == blk.ord)`), which is O(R) per block; `Row.chunks(B)`
  would name it.
