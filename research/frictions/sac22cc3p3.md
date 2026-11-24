# sac22cc3p3 — Bob Sort

## Rounds are a while loop over a power of ten
- **Where:** `port.cp`, `var p = 1 ... while p <= top: ... p *= 10`.
- **What felt wrong:** "for each digit position of the largest value" wants a range of rounds; the
  loop variable is a power of ten stepped by hand.
- **General fix:** a geometric range in std (`powers(10, upto: top)`), or a `digit(x, r)` helper so the loop is `for r in 0..top.digits().len`.
- **Goal it hurts:** express.
- **Severity:** low.

## Dream

```cp
let n = read<1..=1e5>()
var row = read<[n]1..=1e9>()
for p in powers(10, upto: row.max()!):
    row.sort(by: x => x / p % 10)
    print(row)
```
