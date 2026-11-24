# noip01p4

Score: 3/5
Dream: 4/5

Cold review on main 6a989896d: port, dream and friction report, not the C++. The fourth corner as `q + r - p` and `closure()` over every airport pair are the editorial, but the std closure is 20x the C++ (0.58 s vs 0.03 s), measured and reported: a measured perf gap caps the port at 3.

## Checklist failures

- None.

## Owed rewrites

- None.

## Language gaps

- `[V, V] f` from a named fn, and a total-matrix `closure`: friction 1.
- `closure` over `?F64` 20x the C++: friction 2.
