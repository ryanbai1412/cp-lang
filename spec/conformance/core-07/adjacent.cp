// expect: ok, output "8"
// rule: 07.7.5
let a = [1, 4, 9]
print(a.adjacent.map((x, y) => y - x).sum())
