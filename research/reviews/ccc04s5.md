# ccc04s5

Score: 2/5
Dream: 4/5

Cold review against the idiomatic checklist.

- R1: complete, but carries a stale `// Buildable today-form.` line.
- R2: column DP is the editorial, but written as the C++: right-to-left `while`, two
  copies of the vertical scan.
- R4: the scan is written twice (up and down).
- R5: `neg_inf = -1000000000` stands for "unreachable"; `?Int` is the type.
- R6: rows and columns are domains, but every access converts from `Int` (`R(r)`, `C(c)`).
- R10: 1.65x on a 14 ms run.
- Dream: `?Int` states, one scan with `step in [-1, 1]`, `value(ch)`; it still mutates
  `q` by hand (`q += step`), where `R.at(q)` in a `while let` would keep the domain.

## Checklist failures

R2, R4, R5, R6, R10.

## Owed rewrites

Re-port from dream.cp (friction 1).

## Language gaps

None.
