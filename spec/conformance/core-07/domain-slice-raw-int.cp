// expect: E0301
// rule: 07.4.5
domain V = 1..=4
let a = [V] v => 0
let w = a[V(2)..]
print(w[2]) // <- here
