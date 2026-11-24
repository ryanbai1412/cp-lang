// n: 512
// The eliminations, at an n where their n^3 / 64 term dominates the n^2 one.
use std.matrix.{BitMat}
let n: Int = read()
var k = 0
var m = BitMat(n, n)
for i in 0..n:
    m.set(i, i, true)
    for j in 0..n:
        if (i * 131 + j * 71) % 3 == 0: m.set(i, j, true)
let b = [i % 3 == 0 for i in 0..n]
for _ in 0..8:
    if m.det(): k += 1 // timed: matrix.BitMat.det
    k += m.rank() // timed: matrix.BitMat.rank
for _ in 0..6:
    if m.solve(b) != None: k += 1 // timed: matrix.BitMat.solve
    if m.solve_space(b) != None: k += 1 // timed: matrix.BitMat.solve_space
    if m.inverse() != None: k += 1 // timed: matrix.BitMat.inverse
    k += m.mul(m).rows // timed: matrix.BitMat.mul
print(k)
