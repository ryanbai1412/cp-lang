// expect: ok, output "14 2"
// rule: 06.1.4
let ps = [(1, 2), (3, 4)]
let xs = [5, 6, 7]
print(ps.map((a, b) => a * b).sum(), xs.items.count((i, x) => i > 0))
