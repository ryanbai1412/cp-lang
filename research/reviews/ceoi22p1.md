# ceoi22p1

Score: 4/5
Dream: 4/5

Baseline review, cold, on main 96359635. The header (lines 4-13) is a real editorial with the run invariant. The body hand-rolls two stdlib algorithms and drops to `.to_idx` arithmetic for run slices (lines 50-60, 89-97), with a `fixed` flag in the shuffle loop. No dream exists, so the score is capped at 3.

## Checklist failures

- Hand inverse permutation `pos_of` with a `Pos.first` fake initialisation (lines 31-33); `inverse` (`stdlib/seq.cp:576`) exists.
- Hand monotonic stack for next greater (lines 37-44); `next_greater` exists in `stdlib/seq.cp`.
- `.to_idx` / `Pos.nth` arithmetic (lines 53-56, 89-90, 93, 97); the author notes it as WISH A22.
- `fixed` flag with `while not fixed` (lines 70-84).
- `ans` filled with `Card.first` as a fake initialisation (line 71).
- `where` clauses on a `let` (line 21) and on the brute's signature (line 102): check them against §70, which moved filters to `if`.
- The brute hand-merges with three `while` loops (lines 110-122).

## Rewrites owed with today's language

- `let pos_of = deck.inverse()` and `let nxt_gt = deck.next_greater()`.
- Build `ans` as `[Q] qi => ...` over the sorted order, and keep the shuffle state in a `?` or `break` instead of a flag.

## Gaps that need the language

- Range slicing over a domain without `.to_idx` (A22), and `Perm<Card>` / even-`n` input constraints (A21). Both are WISH comments in the port, not a friction report; flag it.

## Cold review (HEAD)

Dream (new) 4/5, port 4/5 (port = dream), cold-reviewed on current main. It checks with zero errors. The header derives the run invariant and the fixed point. `deck.inverse()`, `deck.next_greater()`, a `Fenwick<Card, Int>` and `cur_t: ?Int` carry the algorithm. The run length through `.ord` (line 37) is reported in research/frictions/ceoi22p1.md.
- Minor: `[qi for qi in Q].sorted(by:)` (line 45) is an identity comprehension. Line 65 converts to a raw 1-based `pos1`. The header never bounds the shuffle loop, which runs at most `n` times before the fixed point. Lines 35-36 use comma guards where 4.9 has `and`: `if let … and cond` / `while let … and cond` is the guard spelling (decisions 4.9).
