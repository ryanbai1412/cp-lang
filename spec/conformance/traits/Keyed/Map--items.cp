// expect: ok, output "50"
// engines: next
let m = [1: 10, 2: 20]
var t = 0
for (k, v) in m.items: t += k * v
print(t)
