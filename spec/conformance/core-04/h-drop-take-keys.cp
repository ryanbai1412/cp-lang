// expect: ok, output "20\n10"
// rule: 04a.9.1
let a = [10, 20, 30]
let i = 1
print(a.drop(1)[i])
print(a.take(2)[i - 1])
