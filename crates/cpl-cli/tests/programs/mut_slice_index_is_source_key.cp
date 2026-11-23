// A slice keeps its source's keys, also as a place: `a[1..][1]` is `a[1]`
// to read and to write (R04b.1).
var a = [5, 6, 7]
a[1..][1] = 9
print(a)
var b = [5, 6, 7, 8]
b[1..3][2] += 1
print(b)
var c: [1..=4]Int = 0
c[2..][3] = 7
print(c)
let r = 2..4
b[r][3] = 0
print(b, b[1..][1])
