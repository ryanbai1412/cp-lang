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
  Fresh run (`cpl run`) of `enum Food = M | F | B` then `var t: [?Food]Int = 0`:
  ```
  error[E0204]: an array is indexed by a domain or a length: `[V]Int`, `[n]Int`
  ```
  General fix: an optional of an index type is an index type with one more key; owner: compiler.
- **An enum can't be read from its letter.** The shipments are read as `Char`
  and mapped to codes with an `if` chain. Severity: low.
  Fresh run (`cpl run`) of `enum Food = M | F | B` then `let f: Food = read()`:
  ```
  error[E0302]: reading a `Food` needs to know how its variants are written in the input
  ```
  General fix: io tags on enum variants (single-letter by default); owner: stdlib.
