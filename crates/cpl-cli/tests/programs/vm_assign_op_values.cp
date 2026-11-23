// `a op= b` on boxed values (floats, wide ints, lists, strings) in locals,
// globals, fields and cells runs on the VM and combines as `a = a op b`.
struct Acc:
    var total: F64
    var names: List<Str>
let n: 1..=10 = read()
var f = 0.5
var w: I128 = 1
var xs = [1, 2]
var s = "ab"
var acc = Acc(total: 0.0, names: [])
var grid = [[1], [2]]
var fs = [0.0, 0.0, 0.0]
for i in 0..n:
    f += 1.25
    f *= 1.5
    w *= 3
    w -= 1
    xs += [i]
    s += "x"
    acc.total += 0.5
    acc.names += ["n"]
    grid[i % 2] += [i]
    fs[i % 3] += 2.5
fn bump(k: Int) -> F64:
    var g = 1.0
    var t: List<Int> = []
    for j in 0..k:
        g /= 2.0
        t += [j, j]
    g + F64(t.len)
var hi = 0.0
for i in 0..n:
    hi max= F64(i) * 0.75
print(f, w, xs, s, acc.total, acc.names.len, grid, fs, bump(n), hi)
// A boxed cell of an array of any rank combines in place too.
var bl = ["a", "b"]
bl[0] += "y"
var g: [0..2, 0..2]Str = ""
g[1, 0] = "a"
g[1, 0] += "b"
var h: [0..2, 0..2, 0..2]List<Int> = []
h[1, 0, 1] += [3]
h[1, 0, 1] += [4]
fn cube(k: Int) -> Str:
    var c: [0..k, 0..k, 0..k]Str = ""
    for i in 0..k:
        c[i, 0, i] += "z"
        c[i, 0, i] += "w"
    return c[k - 1, 0, k - 1]
print(bl, g[1, 0], h[1, 0, 1], cube(n))
