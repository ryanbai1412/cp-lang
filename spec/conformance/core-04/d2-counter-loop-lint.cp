// expect: L0605 at 4
// rule: 04a.5.2 04b.1.9
let s = [5, 6, 7, 8]
let w = s[2..4]
var t = 0
for k in 0..w.len: t += w[k]
print(t)
