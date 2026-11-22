// expect: ok, output "6"
// rule: 01.8.2
domain V = 1..=3
let a = [V] v => Int(v)
print(a.sum())
