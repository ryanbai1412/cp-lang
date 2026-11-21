# ioi13p3

Score: pending cold rescore (rewrote both dream and active port with `SegTree<Block, Matrix>` + `Monoid(identity(c), mul)`, Knuth crossing monotonicity & `B = 10` tradeoff in header, tropical `?Int` replacing `1e9` sentinels, 0 lints, `20,604 B`, verified on 15 random grids x 25 updates/queries under ASan+UBSan vs exact DP; `@grader` state object filed in `research/frictions/ioi13p3.md`)
Dream: pending cold rescore (written: `proto/ports/scout_batch_52/dream/ioi13p3.cp`)

Baseline review, cold, on main 96359635. The header (lines 1-4) names the technique, and the tropical `?Int` removes the 1e9 sentinels, which is good. But the segment tree over row blocks is hand-built (`nb + blk`, `o * 2`, `o /= 2`, lines 48-63), and `mul` hides a Knuth-optimisation window (lines 20-32) that the header doesn't mention. No dream exists, so the score is capped at 3.

## Checklist failures

- Hand segment tree over blocks (lines 48-63); `SegTree` takes a custom monoid, and min-plus product is one (associative, not commutative).
- Knuth-style `opt` bounds in `mul` (lines 22-23, 32) used without stating the monotonicity they rely on.
- `B = 10` (line 7) is unexplained (memory versus update cost).
- The power-of-two `while nb * B < r` (line 60) exists only for the hand tree.
- `var w = state!` / `state = w` copy-in, copy-out in every grader call (lines 68-78).

## Rewrites owed with today's language

- `SegTree<Block, Matrix>` with `mul` as the operation, and `set(blk, solve_block(blk))` on update.
- One line in the header for the opt monotonicity, and one for why `B = 10`.

## Gaps that need the language

- Grader-state threading (`?Wombats` global plus copy) is a language gap: there is no `@grader` state object. No friction report exists; flag it.
