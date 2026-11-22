// expect: ok, output "2"
// rule: 12.4.12
let a = [1]
let b = a.clone()
print(a.len + b.len)
