// expect: ok, output "60"
// rule: 13.3.5
let a = [1, 2, 3, 4]
print(a.filter(x => x % 2 == 0).map(x => x * 10).sum())
