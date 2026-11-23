// expect: ok, output "2"
// rule: 01.8.1
let xs = [1, 1, 2, 3, 3]
print(xs.adjacent.count((x, y) => x == y))
