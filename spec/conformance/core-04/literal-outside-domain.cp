// expect: E0902
// rule: 04.2.18 04.8.8
domain V = 1..=4
let a = [V] v => 0
print(a[9]) // <- here
