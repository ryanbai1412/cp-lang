// expect: ok, output "3 4 30 1"
// `[V.ranges] r => …` binds `r` to each piece, and `t[l..r]` with a range literal reads that piece's cell, not a slice.
domain V = 0..5
let l = [V.ranges] r => r.len
var m: [V.ranges]Int = 0
for r in V.ranges:
    m[r] = r.len * 10
let f = [V.ranges] r => r.len * r.len
print(l[V(0)..V(3)], l[V(1)..=V(4)], m[V(0)..V(3)], f[V(4)..=V(4)])
