// expect: ok, output "21"
// rule: 04a.5.2
// A counter loop over a slice walks its keys, not `0..w.len`.
let s = [5, 6, 7, 8]
let w = s[1..4]
var t = 0
for i in w.keys: t += w[i]
print(t)
