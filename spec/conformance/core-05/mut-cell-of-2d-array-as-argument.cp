// expect: ok, output "0 0 0\n0 0 10\n1 12 3"
// `mut g[i, j]` lends one cell of a 2-D array to a `mut` parameter, and the write lands in that cell, as `mut a[i]` does for a list.
fn bump(mut x: Int): x += 10
var g: [0..2, 0..3]Int = 0
var a = [1, 2, 3]
let (i, j) = ([1][0], [2][0])
bump(mut g[i, j])
bump(mut a[i])
for r in 0..2:
    print(g[r, 0], g[r, 1], g[r, 2])
print(a)
