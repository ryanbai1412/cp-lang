// expect: ok, output "12"
// rule: 07.2.8 07.2.9
let a = [1, 2, 3, 4]
print(a.filter(x => x % 2 == 0).map(x => x * 2).sum())
