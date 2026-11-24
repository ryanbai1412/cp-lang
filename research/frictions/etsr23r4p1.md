# Friction Report: etsr23r4p1

## Dream

```cp
for phases in 0..(1 << long.len):
    ...
    if step.running(+).all(s => s >= 0):   // stops at the first negative prefix
        return Some("".join([if s > 0: "(" else: ")" for s in step]))
```

## Gaps

- **A prefix check builds the whole table.** `step.scan(+)` is an eager
  `Prefix`, and `Prefix` is not a `Seq`, so "every prefix sum is non-negative"
  is `V.cuts.all(c => balance[c] >= 0)` over a table built in full for every
  phase choice. The reference stops at the first negative prefix; the port is
  about 10x slower on the max case (0.65 s against 0.06 s). A lazy running fold
  over a `Seq` would match it. Severity: low.
- The statement is not public: `n` is bounded by the reference's int and the
  phase search is exponential in the cycles longer than 2.
