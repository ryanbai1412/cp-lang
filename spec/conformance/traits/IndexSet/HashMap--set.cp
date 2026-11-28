// expect: ok, output "9 1"
// engines: next
var m: HashMap<Int, Int> = HashMap()
m[5] = 7
m[5] = 9
print(m[5], m.len)
