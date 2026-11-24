# ccc19s4

Score: 3/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the block cut DP and both running maxima.
- R2: the O(N) DP walks blocks with raw `(0..n).step(k)` offsets and converts `Spot(i)`/`Spot(j)` at each read (L27-41): index bookkeeping, reported.
- R13: `@alt(most)` keeps the quadratic DP as a checked alternative.
- R5: `k: 1..=n`; scores bounded.
- R10: 0.03 s vs 0.13 s.

## Owed rewrites

None.

## Language gaps

- Blocks of a domain (`Cut.chunks(k)`) and spots reached from cuts (in the report).

## Dream notes

Blocks with `prefix_max`/`suffix_max`, as the editorial; `score[block]` indexes `[Spot]` by a `Cut` range, which only works if the report's cut-to-spot view lands.
