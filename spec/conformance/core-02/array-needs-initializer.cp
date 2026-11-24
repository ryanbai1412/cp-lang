// expect: E0103
// rule: 02.8.3
domain V = 0..3
var a: [V]Int // <- here
print(a[V(0)])
