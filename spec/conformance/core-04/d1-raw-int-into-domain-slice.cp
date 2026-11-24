// expect: E0301
// rule: 04a.5.1 04b.1.11
domain V = 1..=5
let a = [V] v => 0
let i = 0
print(a[V(2)..=V(4)][i]) // <- here
