// n: 256
use std.matrix.{BitMat}
let n: Int = read()
var k = 0
var m = BitMat(n, n)
for i in 0..n:
    m.set(i, i, true)
    for j in 0..n:
        if (i * 131 + j * 71) % 3 == 0: m.set(i, j, true)
let b = [i % 3 == 0 for i in 0..n]
for _ in 0..1 << 15:
    k += BitMat(n, n).rows // timed: matrix.BitMat.new
for _ in 0..4096:
    k += BitMat.identity(n).rows // timed: matrix.BitMat.identity
for i in 0..1 << 18:
    if m.get(i % n, i * 7 % n): k += 1 // timed: matrix.BitMat.get
    m.set(i * 3 % n, i * 5 % n, m.get(i * 5 % n, i * 3 % n)) // timed: matrix.BitMat.set
for _ in 0..32:
    if m.det(): k += 1 // timed: matrix.BitMat.det
    k += m.rank() // timed: matrix.BitMat.rank
for _ in 0..8:
    if m.solve(b) != None: k += 1 // timed: matrix.BitMat.solve
    if m.solve_space(b) != None: k += 1 // timed: matrix.BitMat.solve_space
    if m.inverse() != None: k += 1 // timed: matrix.BitMat.inverse
    k += m.mul(m).rows // timed: matrix.BitMat.mul
print(k)
