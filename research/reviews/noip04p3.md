# noip04p3

Score: 4/5
Dream: 4/5

Cold review on main 6a989896d: port, dream and friction report, not the C++. The port is the dream minus the per-position LIS, which is reported; the header derives `up[i] + down[i] - 1`.

## Checklist failures

- None.

## Owed rewrites

- None.

## Language gaps

- Per-position LIS (`lis_ending`): friction 1 (run added: `error[E0203]: a `[3]Int` has no method `lis_ending``).
