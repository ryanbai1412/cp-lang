// expect: E0203
// rule: 06.1.2
// A list has no `.indices` member: it is indexed by `Int` from 0, so its positions are `0..xs.len`.
let xs = [1, 2]
print(xs.indices) // <- here
