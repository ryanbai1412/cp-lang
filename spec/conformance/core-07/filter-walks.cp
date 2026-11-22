// expect: ok, output "5"
// rule: 07.5.12
let a = [1, 2, 3]
print(a.filter(x => x > 1).sum())
