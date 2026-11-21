# dmopc21c5p5

Score: pending cold rescore (port = dream: replaced nested `if if let` with `dp[nmask, nxt].map(old => cand > old) ?? true`, replaced hand swap loop with `perm[i1..=j].reverse()`, replaced `nxt = 0`/`best_d = -1`/`used` sentinels in `greedy_start` with `argmax` + random tie-break on `rem`, replaced `while two_opt_step: continue` with `loop: if not two_opt_step: break`; 0 lints, `27,514 B`, verified exact n=1..8 and valid n<=100 under ASan+UBSan)
Dream: pending cold rescore (port = dream 1.000x, 0 errors)

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
