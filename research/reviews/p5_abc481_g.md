# p5_abc481_g

Score: 3/5
Dream: 3/5

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

## Cold review (HEAD)

Dream (new) 3/5, port 3/5 (port = dream), cold-reviewed on current main; sent back. It checks with zero errors, and the header derives the gap decomposition and the generating function `G(x)`.
- Owed: `pow2` is a hand table (lines 45-46); `Z(2) ** (n - cur_q)` says it (§70).
- Owed: the gap sweep uses `if t > 0` and `events[t - 1]` (lines 59-63); `events.pairs` (or `.with_prev`) gives the consecutive pair.
- Owed: the final prefix sum (lines 90-96) is a hand accumulator with raw `k - top.p_min` offsets and a bounds check.
- Minor: the sort key maps `End` to `0`/`1` by hand (line 54); enums already sort in declaration order. `add_shifted` (lines 34-38) is the reported `Poly` gap.
- No friction report is needed for the owed items.
