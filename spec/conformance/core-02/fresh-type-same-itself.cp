// expect: ok, output "0"
// rule: 02.9.1
domain A = 1..=3
let a = [A] x => 0
print(a[A.first!])
