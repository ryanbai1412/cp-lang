# abc481f friction

There is no `dream.cp`; the ideal is below.

## Dream
`port.cp` as written. Each group buckets its unvisited out-neighbours by digit
(`var by_digit: [1..=9]List<V>`). Then the buckets in digit order become the next layer's
groups, and a vertex takes `here * 10 + d` the first time it is reached. Writing the bucket
refinement as a pipeline would put the `value[v] = ...` assignment inside a filter
(idiomatic-code/R7), so the explicit loop is the better spelling.

## No gaps
`value: [V]?Z` makes "not reached yet" an absent value, not a sentinel. Digits 1..=9 index
`[1..=9]List<V>` directly. There is no dream/port gap to route.

## Measurements
- Tokens: port 295, original 341.
- `--max` (best of 10): port 0.032 s / 18 MB, `orig.cpp` (not judge-accepted, stress-checked)
  0.107 s / 13 MB. Same output.
- Emitted C++: 20,765 bytes.
