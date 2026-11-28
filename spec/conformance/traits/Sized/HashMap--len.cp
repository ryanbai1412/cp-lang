// expect: ok, output "2"
// engines: next
var m: HashMap<Int, Int> = HashMap()
for i in [1, 2, 1]: m[i] = i
print(m.len)
