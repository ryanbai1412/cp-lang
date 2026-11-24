# canada21p5

Score: 3/5
Dream: 3/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header reduces
the strings to a Markov chain on the distance and uses symmetry for the final division. The
port builds the step matrix and raises it with std `pow`, but writes the matrix product and
identity itself.

## Checklist failures
- idiomatic-code/R3: `fn mul` and `let one` (lines 32-35) are a hand matrix product; in the
  friction (std `Mat` multiplies through `std::function`s, seen in `cpl explain`).
- idiomatic-code/R10: 0.12 s against the C++'s 0.09 s (1.33x, the friction's measurement)
  with no performance friction. Filed in the friction now.

## Owed rewrites
None today.

## Language gaps
- `Mat` over a monomorphised semiring and a matrix `pow` on `[D, D]T`: in the friction.
- 1.33x at `--max`: added to the friction (perf).

## Dream notes
Reads like the editorial, but writes the modulus twice, `Comb<Mod<1e9 + 7>>(n)` and
`[Off, Off]Mod<1e9 + 7>` (caps at 3). Use one `type Z = Mod<P>` as the port does.
