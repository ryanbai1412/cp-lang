// expect: ok, output "0 0 7 0\n0 7 1\n0 0 0 0\n3 9 7 4\n5 2 1 0\n0 3 5"
// rule: 05.3.3 04b.1.12
// A `var` bound to a row or column is a place, as a slice is: writing it, sorting it or reversing it writes the grid's cells.
domain R = 0..3
domain C = 0..4
var g: [R, C]Int = 0
var row = g[R(1)]
row[C(2)] = 7
print(g[R(1)].to_list())
var col = g[.., C(2)]
col[R(0)] = 1
col.reverse()
print(col.to_list())
g[R(1), C(0)] = 5
g[R(1), C(1)] = 9
g[R(1), C(3)] = 4
g[R(0), C(2)] = 0
var bottom = g[R(2)]
for c in C: bottom[c] = 3 - Int(c)
var first = g[.., C(0)]
first.sort()
print([g[r].to_list() for r in R])
print(g[R(0), C(0)], g[R(1), C(0)], g[R(2), C(0)])
