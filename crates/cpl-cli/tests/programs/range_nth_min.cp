use std.seq.{min, max}
domain V = 0..5
let r = V(1)..V(4)
print(min(3, 1), max(4, 9, 2), [5, 2].min()!)
print(Int(r.nth(0)), Int(r.nth(2)))
var a = [V] v => Int(v) * 10
print(a.keys.len, Int(a.keys.nth(3)), a[a.keys.nth(4)])
