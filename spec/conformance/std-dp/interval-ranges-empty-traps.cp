// expect: R0001 (checked)
domain V = 0..4
let t: [V.ranges]Int = 0
print(t[V.range.drop(4)]) // <- here
