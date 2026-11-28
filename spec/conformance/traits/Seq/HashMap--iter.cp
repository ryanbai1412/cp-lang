// expect: ok, output "140"
// engines: next
var m: HashMap<Int, Int> = HashMap()
for i in 1..4: m[i] = i * 10
var s = 0
for (k, v) in m: s += k * v
print(s)
