// expect: ok, output "2 3 12 7\n6 0 1\n2 3\nYES\n0\nYES\nYES\n12.0000000000 4.0000000000 0.0000000000"
use std.matrix.{simplex, sparse_det, sparse_solve}
domain R = 0..2
domain C = 0..3
let m = Mat([R, C] (r, c) => 10 * Int(r) + Int(c))
print(m.rows, m.cols, m.get(1, 2), Mat<Int>.filled(2, 3, 7).get(1, 2))
let e1: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(2)), (1, 1, Mod(3))]
let e0: List<(Int, Int, Mod<998244353>)> = []
let e2: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(1)), (0, 1, Mod(1))]
print(sparse_det(2, e1).v, sparse_det(2, e2).v, sparse_det(0, e0).v)
print(sparse_solve(2, e1, [Mod(4), Mod(9)])!.map(x => x.v))
let e3: List<(Int, Int, Mod<998244353>)> = [(0, 0, Mod(1))]
print(yes(sparse_solve(2, e3, [Mod(1), Mod(0)]) == None))
var bad = 0
for t in 0..60:
    let n = rand(1, 7)
    var e: List<(Int, Int, Mod<998244353>)> = []
    var dense = Mat<Mod<998244353>>.zeros(n, n)
    let cnt = rand(0, 3 * n)
    for _ in 0..cnt:
        let (r, c) = (rand(0, n - 1), rand(0, n - 1))
        let v = Mod<998244353>(rand(-3, 3))
        e.push((r, c, v))
        dense.set(r, c, dense.get(r, c) + v)
    if sparse_det(n, e) != dense.det(): bad += 1
    let b = [Mod<998244353>(rand(0, 9)) for _ in 0..n]
    let got = sparse_solve(n, e, b)
    if dense.det() == Mod<998244353>(0):
        if got != None: bad += 1
    else:
        if got != dense.solve(b): bad += 1
print(bad)
let r = simplex(Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1.0]]), [1.0, 2.0], [1.0, 1.0])!
print(yes(abs(r.val - 3.0) < 1e-9 and abs(r.x[1] - 2.0) < 1e-9))
print(yes(simplex(Mat<F64>.from_rows([[1.0]]), [-1.0], [1.0]) == None and simplex(Mat<F64>.from_rows([[-1.0]]), [0.0], [1.0]) == None))
let r2 = simplex(Mat<F64>.from_rows([[1.0, 1.0], [1.0, 3.0], [-1.0, 0.0]]), [4.0, 6.0, -1.0], [3.0, 2.0])!
print(r2.val, r2.x)
