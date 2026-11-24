# cco03p4

Score: 3/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: `k: Int` (L12) is unbounded; reported.
- R2: subset DP over placed numbers: the editorial.
- R3: `(need[v] - s).is_empty` (L23) spells "subset of" as a difference, and the report asks for `need[v] <= s`, which already exists: `yes(a <= b)` on two `Subset<V>` prints YES under `cpl run`.

## Owed rewrites

- `need[v] <= s`; correct the report.

## Language gaps

- No stated bound for k (in the report).

## Dream notes

Reads like the editorial.
