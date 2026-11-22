// expect: ok, output "30\n30"
// rule: 04a.8.1 04b.1.1
let xs = [0, 10, 20, 30, 40]
domain V = 0..5
let a = [V] v => xs[Int(v)]
let k = 3
print(xs[2..=3][k])
print(a[V(2)..=V(3)][V(3)])
