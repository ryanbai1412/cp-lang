// expect: R0201 (checked)
// rule: 03.1.4
// `V(0)` with `V = 1..=10` is a store into the range, so a default outside it traps.
domain K = 0..5
var c = Counter<K, 1..=10>() // <- here
print(c[K(1)])
