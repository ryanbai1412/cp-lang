// expect: ok, output "3\n7\n4"
// rule: 04a.3.1 04b.1.2
let a = [3, 9, 2, 7, 5]
let m = a[2..5].argmax()!
print(m)
print(a[m])
domain V = 1..=5
let b = [V] v => a[Int(v) - 1]
print(Int(b[V(3)..=V(5)].argmax()!))
