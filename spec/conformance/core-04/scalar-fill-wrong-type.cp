// expect: E0302
// rule: 04.4.27
domain V = 0..3
var a: [V]Int = "x" // <- here
print(a[V(0)])
