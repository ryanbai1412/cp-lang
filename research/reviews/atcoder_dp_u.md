# atcoder_dp_u

Score: 5/5
Dream: 5/5

Cold review against the idiomatic checklist.

- R1: no `limits:` line (header edit); everything else present.
- R2: `score[g]` then `best[s] max= score[g] + best[s - g]` over `s.subsets` is the editorial.
- R3: `Subset`, `.subsets`, `Group.full`; no masks.
- R5: `a: [R, R]-1e9..=1e9`, `n: 1..=16`.
- R10: 0.62x.

## Checklist failures

R1 (`limits:` missing).

## Owed rewrites

None in the code.

## Language gaps

None.
