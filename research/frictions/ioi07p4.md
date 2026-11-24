# Friction Report: ioi07p4

## Dream

```cp
enum Food = M | F | B
var most: [?Food, ?Food, ?Food, ?Food]?Int = None   // an optional enum as an index
let food: [Ship]Food = read()                        // an enum read from its letter
```

## Gaps

- **An optional enum can't index an array.** A mine's last two foods are
  `?Food`, but `[?Food, ...]` is not a table shape, so the DP keys on `0..=3`
  with 0 for "nothing yet" and codes foods 1..=3 by hand. Severity: medium.
- **An enum can't be read from its letter.** The shipments are read as `Char`
  and mapped to codes with an `if` chain. Severity: low.
