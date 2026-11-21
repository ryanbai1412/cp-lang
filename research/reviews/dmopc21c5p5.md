# dmopc21c5p5

Score: pending cold rescore (removed unproved global-optimality claim and unreachable `(n^2)/2 - 1` early exit; header explains prime-parity obstruction and states exact Held-Karp for `n <= 12` vs 2-opt local-optimum guarantee for `n > 12`; `port = dream`, 0 errors, 0 lints)
Dream: pending cold rescore (removed unproved optimality claim and unreachable early exit; states prime-parity obstruction and exact vs 2-opt local-optimum guarantee)

Baseline review, cold, on main 96359635. No editorial: lines 1-3 list prototype deltas. The general case is a random restart plus a 2-opt heuristic with no argument that it reaches the optimum. The body hand-rolls a sieve and an RNG, and drives the search with `-1` sentinels and an `improved` flag. No dream exists, so the score is capped at 3.

## Checklist failures

- No header for why the construction or heuristic is correct (lines 1-3, 72).
- Hand sieve (lines 7-17); `Sieve` (`stdlib/math.cp`) exists.
- Hand LCG `next_rand` (lines 73-76); `std.rand.Rng` exists.
- Held-Karp through `M.from_idx`/`V.from_idx` round trips and `-1` sentinels in `dp`/`prv` (lines 29-64).
- Hand argmax (lines 50-55) and hand reverse with swaps (lines 120-125); `argmax` and `reverse` exist.
- `improved` flag loop with a double `break` (lines 110-127).
- `List.push` loops to build `used`/`best_perm` (lines 84-93) where a comprehension or fill works.

## Rewrites owed with today's language

- `Sieve(n)` for primality, `Rng` for restarts, and `argmax`/`reverse` from `std.seq`.
- State why the construction works in the header, or replace the heuristic with the editorial's construction.

## Gaps that need the language

- None needed: the prototype deltas listed in lines 2-3 (`Sieve`, labeled loops) are available today. The port has no friction report.

## Cold review (HEAD)

Dream (new) 3/5, port 3/5 (port = dream), cold-reviewed on current main; sent back. It checks with zero errors. The header separates exact Held-Karp for `n <= 12` from the 64-start 2-opt search.
- Owed: line 39 is `if if let old = dp[nmask, nxt]: cand > old else: true:`; write `if dp[nmask, nxt].all(old => cand > old):`.
- Owed: the 2-opt move reverses `perm[i1..=j]` by a hand swap loop (lines 60-64); `perm[i1..=j].reverse()` exists.
- Owed: `greedy_start` picks the next vertex with `nxt = 0` and `best_d = -1` sentinels and a raw `used` list of values (lines 69-84). Name the candidate pick (an `argmax` with a random tie-break).
- Minor: the path reconstruction (lines 44-52) is a hand `while let` walk plus `reverse`, and `while two_opt_step(mut perm): continue` (lines 95-96) hides the loop body in the condition.
- No friction report is needed: everything owed exists.

## Cold review (HEAD)

Dream 3/5, port 3/5 (port = dream), rescored cold on current main; sent back. Every rewrite my last pass asked for has landed: the `.map(...) ?? true` test (line 39), `perm[i1..=j].reverse()` (line 60), and an `argmax` with a random tie-break in `greedy_start` (line 70). It checks with zero errors.
- Owed (carried over from the baseline review, which my last pass failed to repeat): for `n > 12` the header calls the 64-start 2-opt search "high-scoring" (lines 9-13) and gives no argument that it reaches the optimum. Either prove it does, or use a construction that does. This is the only item keeping the dream under 4.
- Minor: line 55's comma guard (`if let i1 = i.next, let j_start = i1.next`) is fine as a multi-binding. `loop: if not two_opt_step(mut perm): break` (lines 83-84) is a `while` loop. The path walk (lines 44-51) builds the list and then reverses it.

## Cold review (HEAD)

Port 3/5 and dream 3/5 (port = dream), rescored cold on current main; it goes back. The header now gives an upper bound with an early exit and names the problem as Max-Weight Hamiltonian Path, and `refine_two_opt` is extracted. It checks with zero errors. The obligation from last round is still open:
- **Owed:** lines 14-18 claim that 64 greedy + 2-opt starts "reliably recover the optimum for `n <= 100`", but nothing derives that claim.
  - NP-hardness of the general problem (lines 5-7) says nothing about this particular prime-difference graph.
  - The early-exit bound `(n * n) / 2 - 1` (line 88) ignores the primality condition, so a run may never reach it.
  - Either derive an exact construction for this graph, or state the acceptance rule that the heuristic provably meets.
