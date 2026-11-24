// expect: E0301
// rule: 02.9.1
domain A = 1..=3
domain B = 1..=3
let a = [A] x => 0
print(a[B.first!]) // <- here
