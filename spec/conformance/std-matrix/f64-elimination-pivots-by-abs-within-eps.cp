// expect: ok, output "1 1 1 -2 3 YES"
// rule: std.matrix.Mat.solve std.matrix.Mat.inverse std.matrix.Mat.rank
// The F64 elimination pivots on the largest `abs` entry: a leading 1e-20 would wreck the
// first-nonzero pivot. Without `eps:` only an exact zero is zero.
let a = Mat<F64>.from_rows([[1e-20, 1.0], [1.0, 1.0]])
let x = a.solve([1.0, 2.0], eps: 1e-9)!
let tiny = Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1e-12]])
let inv = Mat<F64>.from_rows([[2.0, 1.0], [4.0, 3.0]]).inverse(eps: 1e-9)!
print(round(x[0]), round(x[1]), tiny.rank(eps: 1e-9), round(inv.get(1, 0)), round(2.0 * inv.get(0, 0)), yes(tiny.rank() == 2))
