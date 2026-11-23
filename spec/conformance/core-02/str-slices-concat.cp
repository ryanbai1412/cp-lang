// expect: ok, output "abz"
// rule: 02.3.3
let a = "abc"
let b = "xyz"
print(a[0..=1] + b[2..])
