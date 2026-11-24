// expect: E0902
// rule: 04.4.25
domain V = 1..=read<1..=5>()
var a: [V]Int = 0
print(a[6]) // <- here
