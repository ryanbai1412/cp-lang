// expect: E0302
// rule: R02.5.5
// A tuple pattern opens a tuple or a record, not a list's or a slice's insides.
let (a, b) = [1, 2] // <- here
print(a, b)
