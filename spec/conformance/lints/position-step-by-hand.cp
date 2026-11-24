// expect: L0101
// rule: 04.3.3
domain V = 1..=3
let v = V(1)
print(V(Int(v) + 1)) // <- here
