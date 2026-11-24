# atcoder_dp_r

Score: 4/5

Cold review against the idiomatic checklist.

- R1: complete.
- R2: "answer = sum of adj^k" is exactly the editorial; `Mat ** k` is the right spelling.
- R3: std `Mat` and `**`, no hand-written squaring loop.
- R5: `k: 1..=1e18`, `adj: [V, V]0..=1`.
- R6: fails mildly: the entries are read back with `0..n` Int indices, not `V`.
- R10: 1.20x on a 13 ms run; reported with an owner.

## Checklist failures

R6 (Int indices into `Mat`), R10 (reported).

## Owed rewrites

Replace the double comprehension with `walks.sum()` once `Mat` has it (Content).

## Language gaps

`Mat` has no domain index and no entry sum (friction 1).
