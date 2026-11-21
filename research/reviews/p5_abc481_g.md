# p5_abc481_g

Score: pending cold rescore (port = dream verbatim: editorial header deriving gap expectation and binomial generating function, prelude `Comb<Z>`, `std.ntt.convolve`, `enum End: Low | High`, `struct Event` / `Gap` / `Segment`, `add_shifted` helper, 0 lints, `31,245 B`, verified on 100 ASan+UBSan cases vs brute force + interpreter agreement)
Dream: pending cold rescore (written: `proto/ports/sim3/dream/p5_abc481_g.cp`, port = dream 1.000x)

Baseline review, cold, on main 96359635. No editorial: the header (lines 1-2) names only the problem and the tools. The sweep encodes endpoint kinds as `0`/`1` ints (lines 26-49), and `dc` (lines 51-68) merges polynomials with binomial shifts nobody derives, so the program can't be checked against the statement. No dream exists, so the score is capped at 3.

## Checklist failures

- No header for the expectation formula or for why the binomial convolutions are right (lines 1-2, 59-60, 71).
- Hand factorial and inverse-factorial tables (lines 11-19); `Comb<Z>` (`stdlib/mod.cp:76`) exists.
- Endpoint kind as a raw `0`/`1` (lines 25-29, 45); use an enum.
- Parallel arrays `ps`/`qs`/`ws` (lines 33-35) and an unnamed triple returned from `dc` (line 51).
- Hand shift-and-add of polynomials (lines 62-67).
- Imports `lib.math.poly` from local examples (line 4) where `std.ntt.convolve` exists.

## Rewrites owed with today's language

- Write the header: the contribution of each gap between consecutive endpoint values and the binomial generating function per segment.
- `Comb<Z>` for the binomials; `enum End: A, B` for the events; a named record for the `dc` result.

## Gaps that need the language

- None found; everything above is today's language. The port has no friction report.
