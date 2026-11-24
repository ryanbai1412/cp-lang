# atcoder_dp_r

Score: 4/5

Cold review against the idiomatic checklist.

- R1: complete.
- R2: "answer = sum of adj^k" is exactly the editorial; `Mat ** k` is the right spelling.
- R3: std `Mat` and `**`, no hand-written squaring loop.
- R5: `k: 1..=1e18`, `adj: [V, V]0..=1`.
- R6: passes: `walks.sum()` reads no entry back by `Int` index; `Mat` itself is not keyed by `V`.
- R10: 0.87x since 7e3ff18e1 (0.96x after 88fdf8bab, 1.17x before).

## Checklist failures

None.

## Owed rewrites

None (`walks.sum()` landed with e9b4b0aa5).

## Language gaps

`Mat` has no domain index (friction 1; waits on Core's E0309).
