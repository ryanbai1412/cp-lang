// expect: E0902
// rule: R04.4.25
// An exclusive literal end past a domain's end is never a cut of it.
domain C = 0..4
let a: [C]Int = [1, 2, 3, 4]
print(a[1..6].len) // <- here
