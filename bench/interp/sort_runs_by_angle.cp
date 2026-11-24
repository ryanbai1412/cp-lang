// cco16p2's shape: n rounds of sorting n directions by angle, then walking
// the runs of equal angle, under `cpl run`.
use std.geometry.{Pt, arg}

let n: Int = read()
var acc = 0
for r in 0..n:
    var ds = [Pt((i * 7919 + r) % 61 - 30, (i * 104729 + 3 * r) % 53 + 1) for i in 0..n]
    ds.sort(by: d => arg(d))
    for line in ds.runs(by: d => arg(d)): acc += line.len * line.len
print(acc)
