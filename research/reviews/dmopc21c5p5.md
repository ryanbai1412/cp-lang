# dmopc21c5p5

Score: pending cold rescore (port = dream verbatim: editorial header explaining exact Held-Karp DP for `n <= 12` and 64-start greedy + 2-opt local search for `n > 12`, prelude `Sieve.new(n)`, `std.rand.Rng`, `Subset<V>` with `?Int`/`?V` and `argmax`, `[Pos]Int` zero-bounds-check `two_opt_step`, 0 lints, `23,309 B`, verified exact optimal `n = 1..8` + ASan/UBSan `n = 9..100` + interpreter agreement)
Dream: pending cold rescore (written: `proto/ports/blind_critic_4/dream/dmopc21c5p5.cp`, port = dream 1.000x)

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
