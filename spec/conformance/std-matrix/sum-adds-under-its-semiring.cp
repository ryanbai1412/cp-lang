// expect: ok, output "10\n0\n4\n2"
// rule: std.matrix.Mat.sum
// `m.sum()` adds every entry with the matrix's own `+`: plain `+` by default, the `over:`
// semiring's add otherwise, so walks counted by a power or the best entry under `max` are one call.
let m = Mat<Int>.from_rows([[1, 2], [3, 4]])
print(m.sum())
print(Mat<Int>.zeros(0, 0).sum())
domain V = 0..2
let adj = Mat([V, V] (i, j) => 1)
print((adj * adj).sum() / 2)
let best = Mat([V, V] (i, j) => if i == j: None else: Some(Int(i) + 2 * Int(j)), over: Semiring(max, +))
print(best.sum()!)
