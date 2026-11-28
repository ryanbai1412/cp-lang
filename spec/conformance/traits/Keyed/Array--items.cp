// expect: ok, output "0 0\n1 10\n2 20"
// engines: next
domain V = 0..3
var a = [V] v => Int(v) * 10
for (k, x) in a.items: print(Int(k), x)
