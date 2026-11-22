// expect: E0302 at 2
// rule: 04.4.27
domain V = 0..3
var a: [V]Int = "x"
print(a[V(0)])
