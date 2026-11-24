# etsr23r2p3

## Dream

```cp
@!multitest
let xs = read<[3]1..=1e9>().sorted()
let step = 1 << (ctz(xs[0]) + 1)
print(yes(xs.windows(2).all(w => (w[1] - w[0]) % step == 0)))
```

## The statement is not in the repo
- **Where:** the header's Problem and the `1..=1e9` bound come from the accepted
  C++ (whose `1 << (ctz + 1)` must stay in `int`), not from the statement, which
  DMOJ serves behind a browser check.
- **General fix:** mirror statements (or at least bounds) into each package.
- **Severity:** medium (the types can't cite their source).

## Two gaps of a sorted triple are spelled out
- **Where:** the `yes(...)` line names both gaps; a pairwise view over a fixed
  array (`windows(2)`) would say "every gap" once.
- **Severity:** low.
