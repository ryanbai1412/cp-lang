// expect: ok, output "12\n1 7 3\n5\n0 9\n4 2"
// `let p = mut a[i]` names a cell: writes through `p` write `a[i]`, and `a` is lent from the
// binding through `p`'s last use, along with any name bound while it is (R05.3.4).
var a = [1, 2, 3]
let p = mut a[1]
p += 10
print(a[1])
fn bump(mut xs: List<Int>, i: Int):
    let c = mut xs[i + 0]
    c = 7
bump(mut a, 1)
print(a)
var g = [[1, 2], [3, 4]]
let q = mut g[1][0]
q += 2
print(g[1][0])
fn two() -> List<Int>:
    var r = [0, 0]
    let k = 1
    let x = mut r[k]
    x = 9
    r
print(two())
var b = [1, 2]
let m = mut b[0]
let was = m
m = was + 3
print(b[0], b[1])
