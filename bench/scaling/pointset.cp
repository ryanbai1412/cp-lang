// n: 4096
use std.geometry.{PointSet, Pt}
let n: Int = read()
let s = PointSet.of([Pt(i, i * 7 % 101) for i in 0..n])
var acc = 0
for i in 0..1 << 18:
    if s.is_empty: acc += i // timed: geometry.PointSet.is_empty
for r in 0..256:
    for k in s.each(): acc += k // timed: geometry.PointSet.each
print(acc)
