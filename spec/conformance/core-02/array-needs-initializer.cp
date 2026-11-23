// expect: E0103 at 2
// rule: 02.8.3
domain V = 0..3
var a: [V]Int
print(a[V(0)])
