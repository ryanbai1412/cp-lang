// expect: ok, output "33 2"
// rule: 07.4.10
let a = [1, 2, 3]
let b = [10, 20]
print(zip(a, b).map((x, y) => x + y).sum(), zip(a, b).len)
