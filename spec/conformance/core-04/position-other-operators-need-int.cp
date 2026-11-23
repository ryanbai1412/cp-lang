// expect: E0309 at 4
// rule: 04.3.3
// Only `v + k`, `v - k` and `v - w` are position arithmetic; `*` takes `Int(v)` first (round 22, H06).
domain V = 0..10
let v = V(7)
print(v * 2)
