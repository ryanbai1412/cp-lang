// expect: ok, output "0 9 1 2 3 4 5 7\n1 2 3 4 5\n9 0 7 3 1 2 4"
// rule: std.seq.insert
var a = [1, 2, 4, 5]
a.insert(0, 0)
a.insert(3, 3)
a.insert(a.len, 7)
a.insert(1, 9)
print(a)
let f = a.remove(0)
let m = a.remove(0)
let e = a.remove(a.len - 1)
print(a)
var b: List<List<Int>> = [[1], [2], [4]]
b.insert(0, [3])
let r = b.remove(b.len - 1)
print(m, f, e, b[0][0], b[1][0], b[2][0], r[0])
