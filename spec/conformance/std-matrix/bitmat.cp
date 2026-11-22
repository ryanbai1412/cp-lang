// expect: ok, output "YES YES YES\nYES YES\nYES YES\nYES YES\nYES YES\nYES YES\nYES YES YES"
use std.matrix.{BitMat, PrefixXorBasis}
var m = BitMat.identity(2)
m.set(0, 1, true)
print(yes(m.det()), yes(m.get(0, 1)), yes(not m.get(1, 0)))
var s = BitMat.new(2, 2)
s.set(0, 0, true)
s.set(0, 1, true)
s.set(1, 0, true)
s.set(1, 1, true)
print(yes(s.rank() == 1), yes(not s.det()))
print(yes(m.solve([true, true]) == Some([false, true])), yes(s.solve([true, false]) == None))
print(yes(s.solve_space([true, true]) == Some((x0: [true, false], basis: [[true, true]]))), yes(BitMat.new(2, 3).rank() == 0))
let inv = m.inverse()!
let p = m.mul(m)
print(yes(inv.get(0, 1) and inv.get(1, 1) and not inv.get(1, 0)), yes(not p.get(0, 1) and p.get(0, 0)))
var big = BitMat.new(130, 130)
for i in 0..130:
    big.set(i, i, true)
    if i + 1 < 130: big.set(i, i + 1, true)
let bi = big.inverse()!
print(yes(big.mul(bi).det() and big.rank() == 130), yes(bi.get(0, 129) and not bi.get(1, 0)))
var px = PrefixXorBasis()
px.push(6)
px.push(5)
px.push(3)
print(yes(px.rank(0..3) == 2 and px.rank(2..3) == 1), yes(px.max_xor(1..3) == 6 and px.max_xor(2..3) == 3), yes(px.contains(1..3, 6) and not px.contains(1..2, 3)))
