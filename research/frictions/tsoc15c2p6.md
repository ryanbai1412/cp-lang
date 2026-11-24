# tsoc15c2p6 — All-Out War!

## Statement bound of an attack's strength not recorded
- **Where:** `port.cp`, `kills: 0..=1e9`.
- **What felt wrong:** the header records `R_i <= 1e9` but not the bound on `c`; `0..=1e9`
  matches the original's `int` and every generated case.
- **Severity:** low.

## Dream

The port, with `print(max(row.fold(l..=r), 0), max(row.all, 0))` written as
`print(row.fold(l..=r).clamp_low(0), ...)` once a `clamp_low`-style spelling exists.
