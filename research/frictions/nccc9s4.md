# Friction report: nccc9s4

## Dream

```cp
let a: [Student]1..=n = read()
let qs: [Q]Range<Student> = read(ranges: 1..=n inclusive)
print_lines(range_mode(a, qs, ties: largest))
```

## One algorithm, not two

The v0.2 port ran two algorithms, chosen by the number of distinct scores: a per-value
prefix count when there were at most 800, rollback Mo's otherwise. Rollback Mo's alone is
O(q B + n^2 / B) for every input, so the rewrite keeps only that.

## Rollback Mo's by hand

std `mo` needs `remove`, and a range mode with ties broken by the largest value has no O(1)
removal. So the port hand-writes rollback Mo's: short queries counted directly, long ones
bucketed by block. A std `mo_rollback(queries, add:, snapshot:, restore:)` would remove the
bucketing code.

## No seed for the running mode

The running best is `?(count, value)` and `add` folds into it with `best ?? (0, v)`. That
avoids a `(0, C.first!)` seed but costs some speed: 0.29 s on the max case against 0.07 s for
the v0.2 port on a loaded box (a plain-tuple seed measured 0.24 s). No time limit is recorded
for the problem.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). Corpus tests
pass.
