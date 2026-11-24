// expect: E0301
// rule: 04.3.1 04.8.1
domain V = 1..=4
let a = [V] v => 0
print(a[V(2).l]) // <- here
