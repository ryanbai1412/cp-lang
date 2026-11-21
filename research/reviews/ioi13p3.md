# ioi13p3

Score: 4/5
Dream: 4/5

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

## Cold review (HEAD)

Dream (new) 4/5, cold-reviewed on current main. The header derives the block size and the Knuth crossing argument. Both errors (`Block` depends on `init`'s `r`, line 51; `self` used in the static `init`, line 67) belong to the stateful `@grader` struct, which is reported in research/frictions/ioi13p3.md.
- Minor: `Matrix = List<List<?Int>>` is indexed by raw columns (line 19). `changeH` and `changeV` have the same body except for `h`/`v` (lines 70-78).

Port 4/5 (new score): it checks with zero errors, is the dream as a stdin command loop, and builds `SegTree<Block, Matrix>` with named `a:`. Owed (minor): lines 37-38 spell `best.all(b => cand < b)` (as in the dream) as `if let … else: true`. The `1`/`2` match arms repeat the leaf rebuild (lines 75-84).
