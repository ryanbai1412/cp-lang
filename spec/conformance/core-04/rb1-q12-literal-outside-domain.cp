// expect: E0902
// rule: 04.2.18
domain V = 1..=4
var a: [V]Int = 0
print(a[9]) // <- here
