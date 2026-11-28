// expect: ok, output "1.0 0.0 0.0 1.0 2.5"
// rule: std.matrix.Mat.identity R08.1.5
// `F64` has ordinary `Zero` and `One` impls, so `Mat<F64>.identity` builds from `T.zero` and `T.one`.
let m = Mat<F64>.identity(2)
print(m.get(0, 0), m.get(0, 1), m.get(1, 0), m.get(1, 1), F64.zero + F64.one * 2.5)
