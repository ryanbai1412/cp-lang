// expect: ok, output "1"
// engines: next
domain V = 0..3
let a: [V]Int = [1, 2, 3]
let b: [V]Int = [1, 2, 4]
print(if a == a.clone() and a != b: 1 else: 0)
