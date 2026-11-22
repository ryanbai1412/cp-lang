// expect: ok, output "1 3 3 0"
// rule: 07.11.1
var a = [3, 1, 2]
a.sort(by: x => desc(x))
a.reverse()
var b = [1, 1, 2, 2, 1]
b.dedup()
let n = b.len
b.fill(0)
print(a[0], a[2], n, b.sum())
