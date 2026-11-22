// expect: E0301 at 4
// rule: 04a.8.1
domain V = 1..=3
let a: [V]Int = [V] v => Int(v)
let xs = List(a)
print(xs[V.first!])
