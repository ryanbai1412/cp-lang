// expect: E0301 at 3
// rule: 02.8.1
domain V = 1..=3
let a = [V] v => 0
print(a[2])
