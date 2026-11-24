// expect: E0323
// rule: 02.6.2
// Two aliases defined by each other never name a type.
type A = B // <- here
type B = A
let x: A = 1
print(x)
