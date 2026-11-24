# noimps22p5

Score: 3/5
Dream: 3/5

Cold review on main 6a989896d: port, dream and friction report, not the C++. Two heap sweeps down the depth levels, one per cost case, as the header derives. The dream covers only `sweep_heavy`, so the score is provisional until the dream is rewritten.

## Checklist failures

- Lines 22-31 build `level` with two hand counters, `depth` and `opened`, both raw `Int` (idiomatic-code/R6, R7).
- Line 61's `d == n - 2` special case is not derived in the header (lines 10-12 describe the size-2 choice only) (idiomatic-code/R1).
- Lines 39-49 keep `least` and `sum` beside the `MaxHeap` by hand; the double-ended heap is reported.

## Owed rewrites

- Derive or remove the `d == n - 2` case in the header.
- Rewrite the dream as the whole program.

## Language gaps

- A double-ended heap with `.min`: friction 1.

## Dream notes

- Covers one of the two sweeps; `sum(for d in 0..n: ...)` puts statements inside an expression, which no proposal in the friction defines.
