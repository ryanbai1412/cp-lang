// expect: ok, output "YES NO 1"
// rule: R04.8.10
// `[V].filled(x)` makes an array over `V`; arrays compare cell by cell, and a list literal
// compared with one takes its type.
domain V = 0..3
var a = [V].filled(1)
a[V(1)] = 2
let b: [V]Int = [1, 2, 1]
let c = [4].filled(1)
print(yes(a == b), yes(a == [1, 1, 1]), c[3])
