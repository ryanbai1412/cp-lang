# dmpg18s5

Score: 2/5
Dream: none

Cold review of `port.cp` (no `dream.cp`, no friction `## Dream`, so capped at 3). Sqrt decomposition with `Pos`, `Val` and `Block` domains and a `[Val, Block]0..=SIDE` counter table (line 29) reads like the header.

## Checklist failures
- idiomatic-code/R3: lines 22-24 build every value's divisor list with a harmonic loop, while the friction says the port uses std `Sieve.divisors`. std has it (`std/math.cp`), so this is bookkeeping std removes, and the friction describes code that is not there.

## Owed rewrites
- Replace lines 22-24 with `Sieve(MAX).divisors(...)` (or keep the table only if a run shows the per-update factorisation is slower, and say so in the friction).
- Fix the friction's description (it names a `[Block, Val]` table; the port has `[Val, Block]`).

## Language gaps
- Speed 0.96 s vs 0.74 s (1.30x) and memory 162 MB vs 99 MB at `--max`: measured in the friction, routed to no owner (idiomatic-code/R10).

## Dream notes
No dream.
