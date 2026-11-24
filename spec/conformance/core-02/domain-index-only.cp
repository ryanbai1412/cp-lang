// expect: E0301
// rule: 02.8.1
domain V = 1..=3
let a = [V] v => 0
let i = 2
print(a[i]) // <- here
