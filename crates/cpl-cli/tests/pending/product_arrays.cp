domain R = 0..2
domain C = 0..3

var g: [(R, C)]Int = 0
g[(R.at(0), C.at(1))] = 7
g[R.at(1), C.at(2)] = 11
print(g[(R.at(0), C.at(1))], g[R.at(1), C.at(2)])

let t: [(R, C)]Int = [(R, C)] (r, c) => r.ord * 10 + c.ord
print(t[(R.at(1), C.at(2))])

let n = 2
let m = 3
var a: [(n, m)]Int = 0
a[(1, 2)] = 42
print(a[1, 2])
