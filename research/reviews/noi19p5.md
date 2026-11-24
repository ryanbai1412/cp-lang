# noi19p5

Score: 2/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. No dream (cap 3), and the friction report says the positions are "refit with std `interpolate` and read with `eval`", but the port hand-writes the three-point Lagrange formula `at` (lines 21-24).

## Checklist failures

- `at` re-derives `lagrange_at` (std/poly.cp: the polynomial through `(i, ys[i])` for `i in 0..ys.len`) by hand (idiomatic-code/R3).
- Lines 32-44 are one nested expression the formatter breaks mid-call (`falling(\n b,\n t - j + 1,\n)`); name the two origin terms.
- 0.26 s vs C++ 0.19 s at max (1.37x) has no performance friction (idiomatic-code/R10).

## Owed rewrites

- `lagrange_at(f, Z(c - 1))` for the query and the refit; correct the friction report.
- Write the dream.

## Language gaps

- None.
